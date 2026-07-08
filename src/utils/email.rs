use crate::errors::IndigoError;

pub struct EmailPayload {
    pub to:      String,
    pub subject: String,
    pub html:    String,
}

pub async fn send_email(
    api_key: &str,
    from:    &str,
    payload: EmailPayload,
) -> Result<(), IndigoError> {
    let body = serde_json::json!({
        "from":    from,
        "to":      [payload.to],
        "subject": payload.subject,
        "html":    payload.html,
    });

    let res = reqwest::Client::new()
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| IndigoError::Email(e.to_string()))?;

    if !res.status().is_success() {
        return Err(IndigoError::Email(format!(
            "Resend error: {}",
            res.status()
        )));
    }
    Ok(())
}

// ── Email templates ────────────────────────────────────────────

pub fn verification_email(name: &str, link: &str) -> String {
    format!(
        r#"<div style="font-family:sans-serif;max-width:520px;margin:auto">
          <h2 style="color:#4f46e5">Welcome to Indigo, {}!</h2>
          <p>Click below to verify your email address.</p>
          <a href="{}"
             style="display:inline-block;padding:12px 28px;
                    background:#4f46e5;color:#fff;border-radius:8px;
                    text-decoration:none;font-weight:600">
            Verify Email
          </a>
          <p style="color:#6b7280;font-size:13px;margin-top:24px">
            Expires in 24 hours.
          </p>
        </div>"#,
        name, link
    )
}

pub fn password_reset_email(name: &str, link: &str) -> String {
    format!(
        r#"<div style="font-family:sans-serif;max-width:520px;margin:auto">
          <h2 style="color:#4f46e5">Reset your Indigo password</h2>
          <p>Hi {}, click below to reset your password.</p>
          <a href="{}"
             style="display:inline-block;padding:12px 28px;
                    background:#4f46e5;color:#fff;border-radius:8px;
                    text-decoration:none;font-weight:600">
            Reset Password
          </a>
          <p style="color:#6b7280;font-size:13px;margin-top:24px">
            Expires in 1 hour.
          </p>
        </div>"#,
        name, link
    )
}

pub fn booking_confirmation_email(
    name:     &str,
    service:  &str,
    date:     &str,
    zoom_url: &str,
) -> String {
    format!(
        r#"<div style="font-family:sans-serif;max-width:520px;margin:auto">
          <h2 style="color:#4f46e5">Booking Confirmed!</h2>
          <p>Hi {}, your <strong>{}</strong> session is booked for
             <strong>{}</strong>.</p>
          <a href="{}"
             style="display:inline-block;padding:12px 28px;
                    background:#0ea5e9;color:#fff;border-radius:8px;
                    text-decoration:none;font-weight:600">
            Join Zoom Meeting
          </a>
        </div>"#,
        name, service, date, zoom_url
    )
}