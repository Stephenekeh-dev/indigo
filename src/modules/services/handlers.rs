use axum::{extract::{State, Path, Query}, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{
    state::AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
    utils::{
        slug::unique_slug,
        zoom::create_meeting,
    
    },
};
use super::models::*;

// ── Services ───────────────────────────────────────────────────

pub async fn list_services(
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<ServiceListing>>> {
    let rows = sqlx::query_as!(
        ServiceListing,
        r#"SELECT id, title, slug, description, short_desc,
                  service_type::text as "service_type!",
                  price_usd::float8 as "price_usd!",
                  duration_hours::float8 as duration_hours,
                  is_active, sort_order, created_at, updated_at
           FROM service_listings
           WHERE is_active = true
           ORDER BY sort_order ASC"#
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get_service(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<ServiceListing>> {
    sqlx::query_as!(
        ServiceListing,
        r#"SELECT id, title, slug, description, short_desc,
                  service_type::text as "service_type!",
                  price_usd::float8 as "price_usd!",
                  duration_hours::float8 as duration_hours,
                  is_active, sort_order, created_at, updated_at
           FROM service_listings
           WHERE slug = $1 AND is_active = true"#,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| IndigoError::NotFound("Service".into()))
}

pub async fn create_service(
    _claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreateServiceDto>,
) -> IndigoResult<Json<ServiceListing>> {
    let id   = Uuid::new_v4();
    let slug = unique_slug(&dto.title, &id);
    let row  = sqlx::query_as!(
        ServiceListing,
        r#"INSERT INTO service_listings
              (id, title, slug, description, short_desc,
               service_type, price_usd, duration_hours)
           VALUES ($1,$2,$3,$4,$5,$6::text::service_type,$7::float8,$8::float8)
           RETURNING id, title, slug, description, short_desc,
                     service_type::text as "service_type!",
                     price_usd::float8 as "price_usd!",
                     duration_hours::float8 as duration_hours,
                     is_active, sort_order, created_at, updated_at"#,
        id, dto.title, slug, dto.description, dto.short_desc,
        dto.service_type, dto.price_usd, dto.duration_hours
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn update_service(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<CreateServiceDto>,
) -> IndigoResult<Json<ServiceListing>> {
    let slug = unique_slug(&dto.title, &id);
    let row  = sqlx::query_as!(
        ServiceListing,
        r#"UPDATE service_listings SET
              title          = $1,
              slug           = $2,
              description    = $3,
              short_desc     = $4,
              service_type   = $5::text::service_type,
              price_usd      = $6::float8,
              duration_hours = $7::float8,
              updated_at     = NOW()
           WHERE id = $8
           RETURNING id, title, slug, description, short_desc,
                     service_type::text as "service_type!",
                     price_usd::float8 as "price_usd!",
                     duration_hours::float8 as duration_hours,
                     is_active, sort_order, created_at, updated_at"#,
        dto.title, slug, dto.description, dto.short_desc,
        dto.service_type, dto.price_usd, dto.duration_hours, id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete_service(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> IndigoResult<Json<serde_json::Value>> {
    sqlx::query!("DELETE FROM service_listings WHERE id = $1", id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({ "message": "Service deleted" })))
}

// ── Bookings ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page:  Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list_my_bookings(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<PaginationQuery>,
) -> IndigoResult<Json<PaginatedResponse<Booking>>> {
    let page   = q.page.unwrap_or(1).max(1);
    let limit  = q.limit.unwrap_or(10).min(50);
    let offset = (page - 1) * limit;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM bookings WHERE client_id = $1",
        claims.sub
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    let rows = sqlx::query_as!(
        Booking,
        r#"SELECT id, service_id, client_id, scheduled_at,
                  duration_minutes, status::text as "status!",
                  zoom_meeting_id, zoom_join_url, zoom_start_url,
                  client_notes, consultant_notes,
                  amount_paid_usd::float8 as amount_paid_usd,
                  stripe_payment_id, created_at, updated_at
           FROM bookings
           WHERE client_id = $1
           ORDER BY scheduled_at DESC
           LIMIT $2 OFFSET $3"#,
        claims.sub, limit, offset
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(PaginatedResponse { data: rows, total, page, limit }))
}

pub async fn create_booking(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreateBookingDto>,
) -> IndigoResult<Json<Booking>> {
    let service = sqlx::query!(
        r#"SELECT title,
                  duration_hours::float8 as duration_hours,
                  price_usd::float8 as "price_usd!"
           FROM service_listings
           WHERE id = $1 AND is_active = true"#,
        dto.service_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("Service".into()))?;

    let duration_mins = (service.duration_hours.unwrap_or(1.0) * 60.0) as u32;
    let start_iso     = dto.scheduled_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Generate Google Meet link — no API call needed
    let meeting = create_meeting(
        "",  // account_id not needed for Google Meet
        "",  // client_id not needed
        "",  // client_secret not needed
        &service.title,
        &start_iso,
        duration_mins,
    )
    .await
    .ok();

    let id = Uuid::new_v4();
    let booking = sqlx::query_as!(
        Booking,
        r#"INSERT INTO bookings
              (id, service_id, client_id, scheduled_at, duration_minutes,
               zoom_meeting_id, zoom_join_url, zoom_start_url, client_notes)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
           RETURNING id, service_id, client_id, scheduled_at,
                     duration_minutes, status::text as "status!",
                     zoom_meeting_id, zoom_join_url, zoom_start_url,
                     client_notes, consultant_notes,
                     amount_paid_usd::float8 as amount_paid_usd,
                     stripe_payment_id, created_at, updated_at"#,
        id,
        dto.service_id,
        claims.sub,
        dto.scheduled_at,
        duration_mins as i32,
        meeting.as_ref().map(|m| m.id.as_str()),
        meeting.as_ref().map(|m| m.join_url.as_str()),
        meeting.as_ref().map(|m| m.start_url.as_str()),
        dto.client_notes
    )
    .fetch_one(&state.db)
    .await?;

    // Send confirmation email — best effort
     match sqlx::query!(
        "SELECT full_name, email FROM users WHERE id = $1",
        claims.sub
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(user)) => {
            tracing::info!("Sending booking email to {}", user.email);
            let date_str = dto.scheduled_at.format("%B %d, %Y at %H:%M UTC").to_string();
            let meet_url = booking.zoom_join_url.clone().unwrap_or_default();
            match crate::utils::email::send_email_smtp(
                &state.config.mail_host,
                state.config.mail_port,
                &state.config.mail_username,
                &state.config.mail_password,
                &state.config.mail_username,
                crate::utils::email::EmailPayload {
                    to:      user.email.clone(),
                    subject: format!("Booking Confirmed — {} (Google Meet)", service.title),
                    html:    crate::utils::email::booking_confirmation_email(
                        &user.full_name,
                        &service.title,
                        &date_str,
                        &meet_url,
                    ),
                },
            ).await {
                Ok(_)  => tracing::info!("Email sent to {}", user.email),
                Err(e) => tracing::error!("Email failed: {:?}", e),
            }
        }
        Ok(None) => tracing::error!("User not found: {}", claims.sub),
        Err(e)   => tracing::error!("DB error: {:?}", e),
    }

    Ok(Json(booking))
}


pub async fn cancel_booking(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> IndigoResult<Json<serde_json::Value>> {
    let affected = sqlx::query!(
        "UPDATE bookings SET status = 'cancelled'
         WHERE id = $1 AND client_id = $2 AND status = 'pending'",
        id, claims.sub
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(IndigoError::NotFound("Booking".into()));
    }
    Ok(Json(serde_json::json!({ "message": "Booking cancelled" })))
}

// ── Projects ───────────────────────────────────────────────────

pub async fn list_my_projects(
    claims: Claims,
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<ClientProject>>> {
    let rows = sqlx::query_as!(
        ClientProject,
        r#"SELECT id, client_id, title, description,
                  service_type::text as "service_type!",
                  status::text as "status!",
                  budget_usd::float8 as budget_usd,
                  created_at, updated_at
           FROM client_projects
           WHERE client_id = $1
           ORDER BY created_at DESC"#,
        claims.sub
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn create_project(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreateProjectDto>,
) -> IndigoResult<Json<ClientProject>> {
    let id  = Uuid::new_v4();
    let row = sqlx::query_as!(
        ClientProject,
        r#"INSERT INTO client_projects
              (id, client_id, title, description, service_type, budget_usd)
           VALUES ($1,$2,$3,$4,$5::text::service_type,$6::float8)
           RETURNING id, client_id, title, description,
                     service_type::text as "service_type!",
                     status::text as "status!",
                     budget_usd::float8 as budget_usd,
                     created_at, updated_at"#,
        id, claims.sub, dto.title, dto.description,
        dto.service_type, dto.budget_usd
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}