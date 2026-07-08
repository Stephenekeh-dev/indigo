use serde::{Deserialize, Serialize};
use crate::errors::IndigoError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoomMeeting {
    pub id:        String,
    pub join_url:  String,
    pub start_url: String,
    pub password:  Option<String>,
}

async fn get_zoom_token(
    account_id:    &str,
    client_id:     &str,
    client_secret: &str,
) -> Result<String, IndigoError> {
    let res = reqwest::Client::new()
        .post("https://zoom.us/oauth/token")
        .query(&[
            ("grant_type",  "account_credentials"),
            ("account_id",  account_id),
        ])
        .basic_auth(client_id, Some(client_secret))
        .send()
        .await
        .map_err(|e| IndigoError::Zoom(e.to_string()))?;

    let body: serde_json::Value = res
        .json()
        .await
        .map_err(|e| IndigoError::Zoom(e.to_string()))?;

    body["access_token"]
        .as_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| IndigoError::Zoom("No access_token in Zoom response".into()))
}

pub async fn create_meeting(
    account_id:       &str,
    client_id:        &str,
    client_secret:    &str,
    topic:            &str,
    start_time:       &str,
    duration_minutes: u32,
) -> Result<ZoomMeeting, IndigoError> {
    let token = get_zoom_token(account_id, client_id, client_secret).await?;

    let body = serde_json::json!({
        "topic":      topic,
        "type":       2,
        "start_time": start_time,
        "duration":   duration_minutes,
        "settings": {
            "host_video":        true,
            "participant_video": true,
            "waiting_room":      true,
            "auto_recording":    "none"
        }
    });

    let res = reqwest::Client::new()
        .post("https://api.zoom.us/v2/users/me/meetings")
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| IndigoError::Zoom(e.to_string()))?;

    let data: serde_json::Value = res
        .json()
        .await
        .map_err(|e| IndigoError::Zoom(e.to_string()))?;

    Ok(ZoomMeeting {
        id:        data["id"].to_string(),
        join_url:  data["join_url"].as_str().unwrap_or("").to_owned(),
        start_url: data["start_url"].as_str().unwrap_or("").to_owned(),
        password:  data["password"].as_str().map(|s| s.to_owned()),
    })
}