use anyhow::{anyhow, bail, Context, Error, Result};
use s3;
use tokio;
use tracing::{debug, error, info, trace, warn};
use ureq;

use crate::clients::bucket_client;
use crate::configparser::{
    config::{ProfileConfig, S3Config},
    get_config, get_profile_config,
};

/// s3 bucket access checks
#[tokio::main(flavor = "current_thread")] // make this a sync function
pub async fn check(profile_name: &str) -> Result<()> {
    let profile = get_profile_config(profile_name)?;

    let bucket = bucket_client(&profile.s3)?;

    if !bucket.exists().await? {
        bail!("bucket {} does not exist!", profile.s3.bucket_name);
    }

    // try uploading file to bucket
    debug!("uploading test file to bucket");
    let test_file = ("/beavercds-test-file", "access test file!");
    bucket
        .put_object_with_content_type(test_file.0, test_file.1.as_bytes(), "text/plain")
        .await
        .with_context(|| {
            format!(
                "could not upload to asset bucket {:?}",
                profile.s3.bucket_name
            )
        })?;

    // download it to check
    debug!("downloading test file");
    let from_bucket = bucket.get_object(test_file.0).await?;
    if from_bucket.bytes() != test_file.1 {
        bail!("uploaded test file contents do not match, somehow!?");
    }

    // download via presigned link like users will
    debug!("downloading test file via presigned link");
    let presigned_url = bucket
        .presign_get(test_file.0, 60, None)
        .await
        .context("failed to fetch presigned url")?;
    trace!("got presigned GET: {presigned_url}");

    let presigned_download = ureq::get(presigned_url)
        .call()
        .context("could not download test file via presigned link")?;
    if presigned_download.into_body().read_to_string()? != test_file.1 {
        bail!("downloaded test file contents do not match, somehow!?");
    }

    // clean up test file after checks
    bucket.delete_object(test_file.0).await?;

    Ok(())
}
