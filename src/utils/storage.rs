use aws_sdk_s3::{
    Client,
    config::{Builder, Credentials, Region},
    primitives::ByteStream,
};
use uuid::Uuid;
use crate::errors::IndigoError;

pub struct R2Config {
    pub account_id:        String,
    pub access_key_id:     String,
    pub secret_access_key: String,
    pub bucket_name:       String,
    pub public_url:        String,
}

fn make_client(cfg: &R2Config) -> Client {
    let creds = Credentials::new(
        &cfg.access_key_id,
        &cfg.secret_access_key,
        None, None, "r2",
    );
    let endpoint = format!(
        "https://{}.r2.cloudflarestorage.com",
        cfg.account_id
    );
    let s3_cfg = Builder::new()
        .endpoint_url(endpoint)
        .credentials_provider(creds)
        .region(Region::new("auto"))
        .force_path_style(true)
        .build();
    Client::from_conf(s3_cfg)
}

pub async fn upload_video(
    cfg:          &R2Config,
    file_bytes:   Vec<u8>,
    original_name: &str,
    course_id:    &str,
) -> Result<String, IndigoError> {
    let ext      = original_name.rsplit('.').next().unwrap_or("mp4");
    let key      = format!("courses/{}/lessons/{}.{}", course_id, Uuid::new_v4(), ext);
    let client   = make_client(cfg);

    let content_type = match ext {
        "mp4"  => "video/mp4",
        "webm" => "video/webm",
        "mov"  => "video/quicktime",
        _      => "video/mp4",
    };

    client
        .put_object()
        .bucket(&cfg.bucket_name)
        .key(&key)
        .body(ByteStream::from(file_bytes))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| IndigoError::Internal(
            anyhow::anyhow!("R2 upload failed: {}", e)
        ))?;

    let public_url = format!("{}/{}", cfg.public_url.trim_end_matches('/'), key);
    tracing::info!("Video uploaded to R2: {}", public_url);
    Ok(public_url)
}

pub async fn upload_image(
    cfg:          &R2Config,
    file_bytes:   Vec<u8>,
    original_name: &str,
    folder:       &str,
) -> Result<String, IndigoError> {
    let ext    = original_name.rsplit('.').next().unwrap_or("jpg");
    let key    = format!("{}/{}.{}", folder, Uuid::new_v4(), ext);
    let client = make_client(cfg);

    let content_type = match ext {
        "png"  => "image/png",
        "webp" => "image/webp",
        "gif"  => "image/gif",
        _      => "image/jpeg",
    };

    client
        .put_object()
        .bucket(&cfg.bucket_name)
        .key(&key)
        .body(ByteStream::from(file_bytes))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| IndigoError::Internal(
            anyhow::anyhow!("R2 upload failed: {}", e)
        ))?;

    Ok(format!("{}/{}", cfg.public_url.trim_end_matches('/'), key))
}