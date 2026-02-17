use anyhow::{anyhow, bail, Context, Error, Result};
use futures::FutureExt;
use itertools::Itertools;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::iter::repeat_with;
use std::path::{self, Path, PathBuf};
use tempfile::tempdir_in;
use tracing::{debug, error, info, trace, warn};
use zip;

use crate::builder::docker;
use crate::clients::docker;
use crate::configparser::challenge::{ChallengeConfig, ProvideConfig};
use crate::utils::TryJoinAll;

/// Extract assets based on provide config to challenge directory - possibly from container.
/// returns extracted file path(s)
///
/// **TOCTOU:** Paths which originated from in-repo assets are validated as present in the
/// challenge directory and readable before return.
/// This is not guaranteed afterwards, and we do not hold handles on the files.
///
/// TODO: Tries to avoid over-writing present files with rename or archival creation by
/// checking for a file just before copy/zip operations.
///
/// TODO: Container-extracted assets are not as strongly validated yet.
pub async fn extract_asset(
    chal: &ChallengeConfig,
    provide: &ProvideConfig,
    profile_name: &str,
) -> Result<Vec<InputPath>> {
    // This needs to handle three cases * 2 sources:
    //   - single or multiple files without renaming (no as: field)
    //   - single file with rename (one item with as:)
    //   - multiple files as archive (multiple items with as:)
    // and whether the file is coming from
    //   - the repo
    //   - or a container

    debug!(
        "extracting assets for challenge {:?} provide {:?}",
        chal.directory, &provide
    );

    let docker = docker().await?;

    let extracted_files = match provide {
        // Repo file paths are relative to the challenge directory, so prepend chal dir

        // Don't need to modify, but still need to validate
        ProvideConfig::FromRepo { files } => process_all_paths(&chal.directory, files)?,
        // These both also ensure we don't over-write anything
        ProvideConfig::FromRepoRename { from, to } => {
            let from = InputPath::validate_single(&chal.directory, from)?;
            let to = OutputPath::validate_single(&chal.directory, to)?;
            std::fs::copy(&from, &to)
                .with_context(|| format!("could not copy repo file {from:?} to {to:?}"))?;
            vec![InputPath(to.0)]
        }
        ProvideConfig::FromRepoArchive {
            files,
            archive_name,
        } => {
            let archive_path = OutputPath::validate_single(&chal.directory, archive_name)?;
            let inputs = process_all_paths(&chal.directory, files)?;
            zip_files(&archive_path.as_ref(), &inputs)
                .with_context(|| format!("could not create archive {archive_name:?}"))?;
            let archive_path = InputPath::validate_single(&chal.directory, &archive_path.0)?;
            vec![archive_path]
        }

        // handle all container events together to manage container, then match again
        ProvideConfig::FromContainer {
            container: container_name,
            ..
        }
        | ProvideConfig::FromContainerRename {
            container: container_name,
            ..
        }
        | ProvideConfig::FromContainerArchive {
            container: container_name,
            ..
        } => {
            let tag = chal.container_tag_for_pod(profile_name, container_name)?;

            let name = format!(
                "asset-container-{}-{}-{}",
                chal.slugify(),
                container_name,
                // include random discriminator to avoid name collisions
                repeat_with(fastrand::alphanumeric)
                    .take(6)
                    .collect::<String>()
            );

            let container = docker::create_container(&tag, &name).await?;

            // match on `provide` enum again to handle each container type
            let files = match provide {
                ProvideConfig::FromContainer {
                    container: container_name,
                    files,
                } => extract_files(chal, &container, files)
                    .await
                    .with_context(|| {
                        format!("could not copy files {files:?} from container {container_name}")
                    }),

                ProvideConfig::FromContainerRename {
                    container: container_name,
                    from,
                    to,
                } => extract_rename(chal, &container, from, &chal.directory.join(to))
                    .await
                    .with_context(|| {
                        format!("could not copy file {from:?} from container {container_name}")
                    }),

                ProvideConfig::FromContainerArchive {
                    container: container_name,
                    files,
                    archive_name,
                } => extract_archive(chal, &container, files, &chal.directory.join(archive_name))
                    .await
                    .with_context(|| {
                        // rustfmt chokes silently if these format args are inlined... ???
                        format!(
                            "could not create archive {:?} with files {:?} from container {}",
                            archive_name, files, container_name
                        )
                    }),

                // non-container variants handled by outer match
                _ => unreachable!(),
            };

            docker::remove_container(container).await?;

            // TODO: FIXME: YIKES: this is just to appease static analysis rn
            // need to do some sort of validation on container extracts
            let files: Vec<InputPath> = files?.into_iter().map(|f| InputPath(f)).collect();
            files
        }
    };

    // assert all files have chal dir prepended
    for path in &extracted_files {
        assert!(
            path.as_ref().starts_with(&chal.directory),
            "extracted path {path:?} for {:?} is missing challenge directory!",
            &chal.directory
        )
    }

    Ok(extracted_files)
}

/// Extract multiple files from container
async fn extract_files(
    chal: &ChallengeConfig,
    container: &docker::ContainerInfo,
    files: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    debug!(
        "extracting {} files without renaming: {:?}",
        files.len(),
        files
    );

    files
        .iter()
        .map(|from| async {
            // use basename of source file as target name
            let to = chal.directory.join(from.file_name().unwrap());
            docker::copy_file(container, from, &to).await
        })
        .try_join_all()
        .await
}

/// Extract one file from container and rename
async fn extract_rename(
    chal: &ChallengeConfig,
    container: &docker::ContainerInfo,
    file: &Path,
    new_name: &Path,
) -> Result<Vec<PathBuf>> {
    debug!("extracting file {:?} renamed to {:?}", file, new_name);

    let new_file = docker::copy_file(container, file, new_name).await?;

    Ok(vec![new_file])
}

/// Extract one or more file from container as archive
async fn extract_archive(
    chal: &ChallengeConfig,
    container: &docker::ContainerInfo,
    files: &[PathBuf],
    archive_name: &Path,
) -> Result<Vec<PathBuf>> {
    debug!(
        "extracting {} files {:?} into archive {:?}",
        files.len(),
        files,
        archive_name
    );

    // copy all listed files to tempdir
    let tempdir = tempfile::Builder::new()
        .prefix(".beavercds-archive-")
        .tempdir_in(".")?;
    let copied_files = files
        .iter()
        .map(|from| async {
            let to = tempdir.path().join(from.file_name().unwrap());
            docker::copy_file(container, from, &to).await
        })
        .try_join_all()
        .await?;

    // TODO: FIXME: YIKES: this is just to appease static analysis rn
    // need to do some sort of validation on container extracts
    let copied_files: Vec<InputPath> = files.into_iter().map(|f| InputPath(f.clone())).collect();
    // archive_name already has the chal dir prepended
    zip_files(archive_name, &copied_files)?;

    Ok(vec![archive_name.to_path_buf()])
}

/// Add multiple local `files` to a zipfile at `zip_name`
pub fn zip_files(archive_name: &Path, files: &[InputPath]) -> Result<PathBuf> {
    debug!("creating zip at {:?}", archive_name);
    let zipfile = File::create(archive_name)?;
    let mut z = zip::ZipWriter::new(zipfile);
    let opts = zip::write::SimpleFileOptions::default();

    let mut buf = vec![];
    for path in files.iter() {
        trace!("adding {:?} to zip", path);
        // TODO: dont read entire file into memory
        File::open(path)?.read_to_end(&mut buf)?;
        // TODO: should this always do basename? some chals might need specific
        // file structure but including dirs should work fine
        z.start_file(path.as_ref().file_name().unwrap().to_string_lossy(), opts)?;
        z.write_all(&buf)?;
        buf.clear();
    }

    z.finish()?;

    Ok(archive_name.to_path_buf())
}

/// Expand globs in all local paths into a single combined array.
/// **Paths must be absolute!**
pub fn expand_all_globs(files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    files.iter().try_fold(Vec::new(), |mut acc, f| {
        let paths = expand_one_glob(f)?;
        acc.extend(paths);
        Ok::<Vec<PathBuf>, anyhow::Error>(acc)
    })
}

/// Expand one path into potentially many by running a glob on local
/// filesystem.
pub fn expand_one_glob(file: &Path) -> Result<Vec<PathBuf>> {
    // FIXME: This should take &str, treating the filename at this point like pathbuf is bad semantics
    // but we're not exporting these bad semantics outside this module to be fair
    let pattern = file.to_string_lossy();
    // TODO: will need more context!
    let paths = glob::glob(&pattern)?.collect::<Result<Vec<_>, _>>()?;
    dbg!(&pattern, &paths);
    Ok(paths)
}

// TODO: verify readability (can stat even if we can't read, duh)

/// For **local** (non-container) asset paths which expands globs
/// but blocks references that are outside the base config directory.
///
/// Recursively descends into folders found as a result of globbing or single specification
/// and accumulates each result as a separate path.
//                                                    Ow! my stack!
///
/// Folders, files, and symlinks to them are accepted. All symlinks will be
/// resolved and replaced in returned paths.
///
/// ---
/// Glob-matching is attempted only on paths which do not exist.
pub fn process_all_paths(chal_dir: &Path, paths: &[PathBuf]) -> Result<Vec<InputPath>> {
    // anything above this dir would be leaving the directory
    let canonical_chal_dir = fs::canonicalize(chal_dir)?;
    let base_cfg_dir = canonical_chal_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow!("Project root not found (CWD is too shallow)"))?;

    let resolved: Vec<InputPath> = paths.iter().try_fold(Vec::new(), |mut acc, path| {
        match std::fs::canonicalize(path) {
            // Assume paths we can resolve aren't globs
            Ok(p) => {
                let stat = fs::metadata(&p)?;
                if stat.is_dir() {
                    // Recursively run on directory contents, hope we don't explode 
                    let dentry_paths = fs::read_dir(&p)?.map(|dr| dr.map(|d| d.path())).collect::<Result<Vec<PathBuf>, _>>()?;
                    acc.extend(process_all_paths(chal_dir, &dentry_paths)?);
                } else if stat.is_file() {
                    acc.push(InputPath::fast_validate_single(&base_cfg_dir, &p)?);
                } else {
                    bail!("Expected canonicalized asset path to be file or directory but {p:?} is not");
                }
                Ok(acc)
            }
            // Try paths we can't locate as globs before failing
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let abs_path = std::path::absolute(path)?;
                let expanded = expand_one_glob(&abs_path)?;
                acc.extend(process_all_paths(chal_dir, &expanded)?);
                Ok(acc)
            }
            Err(e) => Err(e).with_context(|| format!("failed to canonicalize path {:?}", path)),
        }
    })?;
    Ok(resolved)
}

#[derive(Debug, Clone)]
/// A path to a readable file that existed within the base directory.
///
/// Vulnerable to TOCTOU issues. Create and consume as close to usage as possible.
pub struct InputPath(PathBuf);
impl InputPath {
    /// Yield canonical path if the input resolves to one readable file
    /// located within base repo directory
    pub fn validate_single(base: &Path, input: &Path) -> Result<Self> {
        let canonical_base = fs::canonicalize(base)?;
        InputPath::fast_validate_single(&canonical_base, input)
    }

    /// [`validate_single`] but the base path was externally canonicalized
    fn fast_validate_single(canonical_base: &Path, input: &Path) -> Result<Self> {
        let canonical_input = fs::canonicalize(input)?;
        if canonical_input.starts_with(&canonical_base) {
            if fs::metadata(&canonical_input)?.is_file() {
                Ok(Self(canonical_input))
            } else {
                bail!("Input path {input:?} exists within base config directory, but does not appear to be a normal file")
            }
        } else {
            bail!(
                "Input path {input:?} does not appear to be within base config directory {canonical_base:?}\n
                    successfuly canonicalized into {canonical_input:?}"
            )
        }
    }
}
// FIXME: this was a quick hack but other semantics might be preferable
// need to make it easy to expose internal for exernal APIs
impl AsRef<Path> for InputPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone)]
/// A canonical path within the base directory which had no detectable object,
/// but did have a parent directory we can write in.
///
/// Vulnerable to TOCTOU issues. Create and consume as close to usage as possible.
pub struct OutputPath(PathBuf);
impl OutputPath {
    /// Yield canonical path if the parent directory is within base,
    /// writable, and no object is present
    // TODO: I got lazy and prompted this one, I hate the verbosity but it might be better for readability tbh. Normalize styles?
    // functionality also needs to be audited more carefully b/c we can't canonicalize a path that doesn't exist! path::absolute not the same
    pub fn validate_single(base: &Path, output: &Path) -> Result<Self> {
        let canonical_base = fs::canonicalize(base)?;
        let abs_output = std::path::absolute(output)?;

        // any file at this path is a no-no
        if fs::symlink_metadata(&abs_output).is_ok() {
            bail!("Output path {output:?} already exists");
        }

        let parent = abs_output
            .parent()
            .ok_or_else(|| anyhow!("Output path {output:?} has no parent directory"))?;

        let canonical_parent = fs::canonicalize(parent)
            .with_context(|| format!("Parent directory {parent:?} does not exist"))?;

        if !canonical_parent.starts_with(&canonical_base) {
            bail!("Output path parent {canonical_parent:?} is outside base directory {canonical_base:?}");
        }

        if fs::metadata(&canonical_parent)?.permissions().readonly() {
            bail!("Parent directory {canonical_parent:?} is read-only");
        }

        let file_name = abs_output
            .file_name()
            .ok_or_else(|| anyhow!("Output path {output:?} invalid filename"))?;

        Ok(Self(canonical_parent.join(file_name)))
    }
}
impl AsRef<Path> for OutputPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
