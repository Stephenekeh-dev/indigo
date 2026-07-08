use axum::{extract::{State, Path, Query}, Json};
use serde::Deserialize;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use crate::{
    state::AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
    utils::{
        slug::unique_slug,
        zoom::create_meeting,
        email::{send_email, EmailPayload, booking_confirmation_email},
    },
};
use super::models::*;

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page:  Option<i64>,
    pub limit: Option<i64>,
}

// ── Service listings ───────────────────────────────────────────

pub async fn list_services(
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<ServiceListing>>> {
    let rows = sqlx::query_as!(
        ServiceListing,
        r#"SELECT id, title, slug, description, short_desc,
                  service_type::text as "service_type!",
                  price_usd, duration_hours,
                  is_active, sort_order,
                  created_at, updated_at
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
                  price_usd, duration_hours,
                  is_active, sort_order,
                  created_at, updated_at
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
    let id       = Uuid::new_v4();
    let slug     = unique_slug(&dto.title, &id);
    let price    = BigDecimal::from_str(&dto.price_usd.to_string()).unwrap_or_default();
    let duration = dto.duration_hours
        .map(|h| BigDecimal::from_str(&h.to_string()).unwrap_or_default());

    let row = sqlx::query_as!(
        ServiceListing,
        r#"INSERT INTO service_listings
              (id, title, slug, description, short_desc,
               service_type, price_usd, duration_hours)
           VALUES ($1,$2,$3,$4,$5,$6::text::service_type,$7,$8)
           RETURNING id, title, slug, description, short_desc,
                     service_type::text as "service_type!",
                     price_usd, duration_hours,
                     is_active, sort_order,
                     created_at, updated_at"#,
        id, dto.title, slug, dto.description, dto.short_desc,
        dto.service_type, price, duration
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

// ── Bookings ───────────────────────────────────────────────────

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
                  amount_paid_usd, stripe_payment_id,
                  created_at, updated_at
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
                  price_usd::float8 as price_usd
           FROM service_listings
           WHERE id = $1 AND is_active = true"#,
        dto.service_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("Service".into()))?;

    let duration_mins = (service.duration_hours.unwrap_or(1.0) * 60.0) as u32;
    let start_iso     = dto.scheduled_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let zoom = create_meeting(
        &state.config.zoom_account_id,
        &state.config.zoom_client_id,
        &state.config.zoom_client_secret,
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
                     amount_paid_usd, stripe_payment_id,
                     created_at, updated_at"#,
        id, dto.service_id, claims.sub, dto.scheduled_at, duration_mins as i32,
        zoom.as_ref().map(|z| z.id.as_str()),
        zoom.as_ref().map(|z| z.join_url.as_str()),
        zoom.as_ref().map(|z| z.start_url.as_str()),
        dto.client_notes
    )
    .fetch_one(&state.db)
    .await?;

    if let Ok(user) = sqlx::query!(
        "SELECT full_name, email FROM users WHERE id = $1",
        claims.sub
    )
    .fetch_one(&state.db)
    .await
    {
        let date_str = dto.scheduled_at.format("%B %d, %Y at %H:%M UTC").to_string();
        let zoom_url = booking.zoom_join_url.clone().unwrap_or_default();
        let _ = send_email(
            &state.config.resend_api_key,
            &state.config.email_from,
            EmailPayload {
                to:      user.email,
                subject: format!("Booking Confirmed — {}", service.title),
                html:    booking_confirmation_email(
                    &user.full_name,
                    &service.title,
                    &date_str,
                    &zoom_url,
                ),
            },
        )
        .await;
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
                  budget_usd, created_at, updated_at
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
    let id         = Uuid::new_v4();
    let budget     = dto.budget_usd
        .map(|b| BigDecimal::from_str(&b.to_string()).unwrap_or_default());

    let row = sqlx::query_as!(
        ClientProject,
        r#"INSERT INTO client_projects
              (id, client_id, title, description, service_type, budget_usd)
           VALUES ($1,$2,$3,$4,$5::text::service_type,$6)
           RETURNING id, client_id, title, description,
                     service_type::text as "service_type!",
                     status::text as "status!",
                     budget_usd, created_at, updated_at"#,
        id, claims.sub, dto.title, dto.description,
        dto.service_type, budget
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}