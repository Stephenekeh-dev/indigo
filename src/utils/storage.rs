use crate::errors::IndigoError;

/// Upload a file to Cloudflare R2 (S3-compatible)
/// For production use aws-sdk-s3 with custom endpoint
pub async fn upload_file(
    account_id:   &str,
    _access_key:  &str,
    _secret_key:  &str,
    _bucket:      &str,
    key:          &str,
    data:         Vec<u8>,
    content_type: &str,
) -> Result<String, IndigoError> {
    let url = format!(
        "https://{}.r2.cloudflarestorage.com/indigo-assets/{}",
        account_id, key
    );

    reqwest::Client::new()
        .put(&url)
        .header("Content-Type", content_type)
        .body(data)
        .send()
        .await
        .map_err(|e| {
            IndigoError::Internal(anyhow::anyhow!("Upload failed: {}", e))
        })?;

    Ok(format!("https://assets.indigo.dev/{}", key))
}