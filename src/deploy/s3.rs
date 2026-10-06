use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Error, Ok, Result};
use futures::future::try_join_all;
use itertools::Itertools;
use s3::Bucket;
use tokio;
use tracing::{debug, error, info, trace, warn};
use url::Url;

use crate::builder::BuildResult;
use crate::clients::bucket_client;
use crate::configparser::config::ProfileConfig;
use crate::configparser::{enabled_challenges, get_config, get_profile_config, ChallengeConfig};
use crate::utils::TryJoinAll;

/// Artifacts and information about a deployed challenges.
pub struct S3DeployResult {
    /// Presigned download URL used by frontend
    pub presigned_asset_urls: Vec<String>,
}

/// Upload all asset files for a challenge to the chal assets bucket,
/// Returns presigned urls of upload files for access.
pub async fn upload_challenge_assets(
    profile_name: &str,
    chal: &ChallengeConfig,
    build_result: &BuildResult,
) -> Result<S3DeployResult> {
    let profile = get_profile_config(profile_name)?;
    let enabled_challenges = enabled_challenges(profile_name)?;

    let bucket = bucket_client(&profile.s3)?;

    info!("uploading assets for chal {:?}...", chal.directory);

    // Upload each asset and collect the public url for each object.
    let uploaded = build_result
        .assets
        .iter()
        .map(|asset_file| async move {
            debug!("uploading file {:?}", asset_file);
            // Upload file to the bucket
            let path_in_bucket = upload_single_file(bucket, chal, asset_file)
                .await
                .with_context(|| format!("failed to upload file {asset_file:?}"))?;

            // Generate a presigned url with enough expiry to last through a
            // usual event (1 week).
            // const DURATION = time::Duration::from_weeks(1).as_secs();
            const EXPIRY: u32 = 7 * 60 * 60;
            let presigned_url = bucket
                .presign_get(path_in_bucket.to_string_lossy(), EXPIRY, None)
                .await
                .context("failed to fetch presigned url")?;
            trace!("got presigned GET: {presigned_url}");

            Ok(presigned_url)
        })
        .try_join_all()
        .await
        .with_context(|| format!("failed to upload asset files for chal {:?}", chal.directory))?;

    // return new BuildResult with assets as bucket path
    Ok(S3DeployResult {
        presigned_asset_urls: uploaded,
    })
}

/// Upload a single file to a bucket.
async fn upload_single_file(
    bucket: &Bucket,
    chal: &ChallengeConfig,
    file: &Path,
) -> Result<PathBuf> {
    // e.g. s3.example.domain/assets/misc/foo/stuff.zip
    let path_in_bucket = format!(
        "assets/{chal_slug}/{file}",
        chal_slug = chal.directory.to_string_lossy(),
        file = file.file_name().unwrap().to_string_lossy()
    );

    trace!("uploading {:?} to bucket path {:?}", file, &path_in_bucket);

    // TODO: move to async/streaming to better handle large files and report progress
    let mut asset_file = tokio::fs::File::open(file).await?;
    let r = bucket
        .put_object_stream(&mut asset_file, &path_in_bucket)
        .await?;
    trace!("uploaded {} bytes for file {:?}", r.uploaded_bytes(), file);

    Ok(PathBuf::from(path_in_bucket))
}
