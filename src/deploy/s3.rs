use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Error, Ok, Result};
use futures::future::try_join_all;
use itertools::Itertools;
use s3::Bucket;
use tokio;
use tracing::{debug, error, info, trace, warn};
use url::Url;

use crate::builder::artifacts;
use crate::builder::BuildResult;
use crate::clients::bucket_client;
use crate::configparser::config::ProfileConfig;
use crate::configparser::{enabled_challenges, get_config, get_profile_config, ChallengeConfig};
use crate::utils::TryJoinAll;

/// Artifacts and information about a deployed challenges.
pub struct S3DeployResult {
    pub uploaded_asset_urls: Vec<String>,
}

/// Upload files to frontend asset bucket
/// Returns urls of upload files.
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

            // S3 API does not have a method to get the public URL, but does
            // have one to create a presigned URL. We need the server to give us
            // the correct URL since we can't reliably assume the format of the
            // full URL for non-AWS storage providers that all have different
            // formats for combining the endpoint and region.
            //
            // Generate a presigned url with expiry in one second, just to make
            // sure this can't be used.
            let presigned_url = bucket
                .presign_get(path_in_bucket.to_string_lossy(), 1, None)
                .await
                .context("failed to fetch presigned url")?;
            trace!("got temporary presigned GET: {presigned_url}");

            // Strip off the signing parameters to get the public object url
            let mut url = Url::parse(&presigned_url)?;
            url.set_query(None);

            Ok(url.to_string())
        })
        .try_join_all()
        .await
        .with_context(|| format!("failed to upload asset files for chal {:?}", chal.directory))?;

    // return new BuildResult with assets as bucket path
    Ok(S3DeployResult {
        uploaded_asset_urls: uploaded,
    })
}

async fn upload_single_file(
    bucket: &Bucket,
    chal: &ChallengeConfig,
    file: &artifacts::InputPath,
) -> Result<PathBuf> {
    // e.g. s3.example.domain/assets/misc/foo/stuff.zip
    let path_in_bucket = format!(
        "assets/{chal_slug}/{file}",
        chal_slug = chal.directory.to_string_lossy(),
        file = file.as_ref().file_name().unwrap().to_string_lossy()
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
