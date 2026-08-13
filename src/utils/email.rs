use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use crate::errors::IndigoError;

pub struct EmailPayload {
    pub to:      String,
    pub subject: String,
    pub html:    String,
}

pub async fn send_email(
    api_key:  &str,
    from:     &str,
    payload:  EmailPayload,
) -> Result<(), IndigoError> {
    // This function signature kept for compatibility
    // but we now use Gmail SMTP via config
    // Call send_email_smtp directly where possible
    tracing::warn!("send_email called without SMTP config — email not sent to {}", payload.to);
    Ok(())
}

pub async fn send_email_smtp(
    host:     &str,
    port:     u16,
    username: &str,
    password: &str,
    from:     &str,
    payload:  EmailPayload,
) -> Result<(), IndigoError> {
    if username.is_empty() {
        tracing::warn!("SMTP not configured — skipping email to {}", payload.to);
        return Ok(());
    }

    let from_addr = format!("Indigo <{}>", from);
    let message   = Message::builder()
        .from(from_addr.parse().map_err(|e| IndigoError::Email(format!("Invalid from: {}", e)))?)
        .to(payload.to.parse().map_err(|e| IndigoError::Email(format!("Invalid to: {}", e)))?)
        .subject(payload.subject)
        .header(ContentType::TEXT_HTML)
        .body(payload.html)
        .map_err(|e| IndigoError::Email(e.to_string()))?;

    let creds = Credentials::new(username.to_owned(), password.to_owned());

    let mailer = if port == 465 {
        SmtpTransport::relay(host)
            .map_err(|e| IndigoError::Email(e.to_string()))?
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::starttls_relay(host)
            .map_err(|e| IndigoError::Email(e.to_string()))?
            .credentials(creds)
            .build()
    };

    tokio::task::spawn_blocking(move || {
        mailer.send(&message)
    })
    .await
    .map_err(|e| IndigoError::Email(e.to_string()))?
    .map_err(|e| IndigoError::Email(e.to_string()))?;

    tracing::info!("Email sent to {}", payload.to);
    Ok(())
}

// ── Base template ──────────────────────────────────────────────

fn base_template(title: &str, content: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>{title}</title>
</head>
<body style="margin:0;padding:0;background:#f8fafc;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif">
  <table width="100%" cellpadding="0" cellspacing="0">
    <tr>
      <td align="center" style="padding:40px 20px">
        <table width="560" cellpadding="0" cellspacing="0"
               style="background:#ffffff;border-radius:16px;overflow:hidden;box-shadow:0 4px 24px rgba(0,0,0,0.06)">
          <tr>
            <td style="background:linear-gradient(135deg,#4f46e5,#7c3aed);padding:28px 40px">
              <span style="color:#fff;font-size:22px;font-weight:800">◆ Indigo</span>
            </td>
          </tr>
          <tr>
            <td style="padding:36px 40px">
              {content}
            </td>
          </tr>
          <tr>
            <td style="padding:20px 40px;border-top:1px solid #f1f5f9;background:#fafafa;text-align:center">
              <p style="margin:0;font-size:12px;color:#94a3b8">
                © 2026 Indigo · The Rust Consulting Platform
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#)
}

fn btn(url: &str, label: &str) -> String {
    format!(r#"<a href="{url}" style="display:inline-block;padding:13px 28px;background:#4f46e5;color:#fff;border-radius:8px;text-decoration:none;font-weight:700;font-size:15px">{label}</a>"#)
}

// ── Templates ──────────────────────────────────────────────────

pub fn verification_email(name: &str, link: &str) -> String {
    let content = format!(r#"
      <h1 style="margin:0 0 12px;font-size:24px;font-weight:800;color:#0f172a">Welcome to Indigo, {name}! 🦀</h1>
      <p style="margin:0 0 24px;font-size:15px;color:#475569;line-height:1.7">
        Verify your email address to unlock bookings, courses, and the AI assistant.
      </p>
      {btn}
      <p style="margin:20px 0 0;font-size:13px;color:#94a3b8">Link expires in 24 hours.</p>
    "#, name=name, btn=btn(link, "Verify Email Address"));
    base_template("Welcome to Indigo", &content)
}

pub fn password_reset_email(name: &str, link: &str) -> String {
    let content = format!(r#"
      <h1 style="margin:0 0 12px;font-size:24px;font-weight:800;color:#0f172a">Reset your password</h1>
      <p style="margin:0 0 24px;font-size:15px;color:#475569;line-height:1.7">
        Hi {name}, click below to reset your Indigo password.
      </p>
      {btn}
      <p style="margin:20px 0 0;font-size:13px;color:#94a3b8">Expires in 1 hour.</p>
    "#, name=name, btn=btn(link, "Reset Password"));
    base_template("Reset your Indigo password", &content)
}

pub fn booking_confirmation_email(
    name:     &str,
    service:  &str,
    date:     &str,
    meet_url: &str,
) -> String {
    let meet_btn = if !meet_url.is_empty() {
        format!(r#"<a href="{meet_url}" style="display:inline-block;padding:13px 28px;background:#1a73e8;color:#fff;border-radius:8px;text-decoration:none;font-weight:700;font-size:15px">Join Google Meet</a>"#)
    } else {
        String::new()
    };

    let content = format!(r#"
      <h1 style="margin:0 0 12px;font-size:24px;font-weight:800;color:#0f172a">Booking Confirmed! ✅</h1>
      <p style="margin:0 0 20px;font-size:15px;color:#475569">Hi {name}, your session is confirmed.</p>
      <table width="100%" cellpadding="0" cellspacing="0"
             style="background:#f0fdf4;border:1px solid #bbf7d0;border-radius:10px;margin-bottom:24px">
        <tr><td style="padding:20px">
          <p style="margin:0 0 8px;font-size:14px;color:#374151"><strong>Service:</strong> {service}</p>
          <p style="margin:0 0 8px;font-size:14px;color:#374151"><strong>Date & Time:</strong> {date}</p>
          <p style="margin:0 0 8px;font-size:14px;color:#374151"><strong>Platform:</strong> Google Meet</p>
          <p style="margin:0;font-size:14px;color:#374151">
            <strong>Link:</strong> <a href="{meet_url}" style="color:#4f46e5">{meet_url}</a>
          </p>
        </td></tr>
      </table>
      {meet_btn}
    "#, name=name, service=service, date=date, meet_url=meet_url, meet_btn=meet_btn);
    base_template("Booking Confirmed — Indigo", &content)
}

pub fn enrollment_confirmation_email(
    name:       &str,
    course:     &str,
    course_url: &str,
) -> String {
    let content = format!(r#"
      <h1 style="margin:0 0 12px;font-size:24px;font-weight:800;color:#0f172a">You're enrolled! 📚</h1>
      <p style="margin:0 0 20px;font-size:15px;color:#475569;line-height:1.7">
        Hi {name}, you have been enrolled in <strong>{course}</strong>.
        Lifetime access — learn at your own pace.
      </p>
      {btn}
    "#, name=name, course=course, btn=btn(course_url, "Start Learning →"));
    base_template(&format!("Enrolled in {}", course), &content)
}

pub fn order_confirmation_email(
    name:      &str,
    reference: &str,
    amount:    &str,
    items:     &str,
) -> String {
    let content = format!(r#"
      <h1 style="margin:0 0 12px;font-size:24px;font-weight:800;color:#0f172a">Order Confirmed! 🛒</h1>
      <p style="margin:0 0 20px;font-size:15px;color:#475569">Hi {name}, your payment was successful.</p>
      <table width="100%" cellpadding="0" cellspacing="0"
             style="background:#f0fdf4;border:1px solid #bbf7d0;border-radius:10px;margin-bottom:24px">
        <tr><td style="padding:20px">
          <p style="margin:0 0 8px;font-size:14px;color:#374151"><strong>Reference:</strong> {reference}</p>
          <p style="margin:0 0 8px;font-size:14px;color:#374151"><strong>Amount:</strong> {amount}</p>
          <p style="margin:0 0 8px;font-size:14px;color:#374151"><strong>Items:</strong> {items}</p>
          <p style="margin:0;font-size:14px;color:#15803d;font-weight:700">Status: ✅ Paid</p>
        </td></tr>
      </table>
      {btn}
    "#, name=name, reference=reference, amount=amount, items=items,
    btn=btn("http://localhost:4200/dashboard", "Go to Dashboard →"));
    base_template("Order Confirmed — Indigo", &content)
}

pub fn newsletter_confirm_email(name: &str, confirm_link: &str) -> String {
    let content = format!(r#"
      <h1 style="margin:0 0 12px;font-size:24px;font-weight:800;color:#0f172a">Confirm your subscription 📬</h1>
      <p style="margin:0 0 24px;font-size:15px;color:#475569;line-height:1.7">
        Hi {name}, click below to confirm your Indigo newsletter subscription.
      </p>
      {btn}
    "#, name=name, btn=btn(confirm_link, "Confirm Subscription"));
    base_template("Confirm your Indigo newsletter", &content)
}