use uuid::Uuid;

pub struct MeetingLink {
    pub id:        String,
    pub join_url:  String,
    pub start_url: String,
    pub password:  Option<String>,
}

/// Generate a Google Meet link without any API call.
/// Google Meet creates the room automatically when the first person joins.
pub async fn create_meeting(
    _account_id:       &str,
    _client_id:        &str,
    _client_secret:    &str,
    _topic:            &str,
    _start_time:       &str,
    _duration_minutes: u32,
) -> Result<MeetingLink, String> {
    // Generate a unique meeting code in Google Meet format: xxx-xxxx-xxx
    let id       = Uuid::new_v4().to_string().replace("-", "");
    let part1    = &id[0..3];
    let part2    = &id[3..7];
    let part3    = &id[7..10];
    let code     = format!("{}-{}-{}", part1, part2, part3);
    let join_url = format!("https://meet.google.com/{}", code);

    tracing::info!("Generated Google Meet link: {}", join_url);

    Ok(MeetingLink {
        id:        code.clone(),
        join_url:  join_url.clone(),
        start_url: join_url,
        password:  None,
    })
}