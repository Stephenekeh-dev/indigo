use axum::{extract::{State, Path}, Json};
use uuid::Uuid;
use crate::{
    AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
    utils::slug::unique_slug,
};
use super::models::*;

pub async fn list_events(
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<Event>>> {
    let rows = sqlx::query_as!(
        Event,
        r#"SELECT id, title, slug, description,
                  event_type::text as "event_type!",
                  status::text as "status!",
                  is_online, is_free, price_usd::float8,
                  max_attendees, scheduled_at, duration_minutes,
                  timezone, zoom_join_url, thumbnail_url, tags,
                  created_at, updated_at
           FROM events
           WHERE status IN ('scheduled','live')
             AND scheduled_at > NOW()
           ORDER BY scheduled_at ASC"#
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get_event(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<Event>> {
    sqlx::query_as!(
        Event,
        r#"SELECT id, title, slug, description,
                  event_type::text as "event_type!",
                  status::text as "status!",
                  is_online, is_free, price_usd::float8,
                  max_attendees, scheduled_at, duration_minutes,
                  timezone, zoom_join_url, thumbnail_url, tags,
                  created_at, updated_at
           FROM events WHERE slug = $1"#,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| IndigoError::NotFound("Event".into()))
}

pub async fn create_event(
    _claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreateEventDto>,
) -> IndigoResult<Json<Event>> {
    let id      = Uuid::new_v4();
    let slug    = unique_slug(&dto.title, &id);
    let tz      = dto.timezone.unwrap_or_else(|| "UTC".into());
    let is_free = dto.is_free.unwrap_or(true);
    let row = sqlx::query_as!(
        Event,
        r#"INSERT INTO events
              (id, title, slug, description, event_type,
               scheduled_at, duration_minutes, is_free,
               price_usd, max_attendees, timezone)
           VALUES ($1,$2,$3,$4,$5::event_type,$6,$7,$8,$9,$10,$11)
           RETURNING id, title, slug, description,
                     event_type::text as "event_type!",
                     status::text as "status!",
                     is_online, is_free, price_usd::float8,
                     max_attendees, scheduled_at, duration_minutes,
                     timezone, zoom_join_url, thumbnail_url, tags,
                     created_at, updated_at"#,
        id, dto.title, slug, dto.description, dto.event_type,
        dto.scheduled_at, dto.duration_minutes, is_free,
        dto.price_usd, dto.max_attendees, tz
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn register_for_event(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<RegisterEventDto>,
) -> IndigoResult<Json<serde_json::Value>> {
    let exists = sqlx::query_scalar!(
        "SELECT id FROM event_registrations WHERE event_id = $1 AND user_id = $2",
        dto.event_id, claims.sub
    )
    .fetch_optional(&state.db)
    .await?;

    if exists.is_some() {
        return Err(IndigoError::Conflict("Already registered for this event".into()));
    }

    sqlx::query!(
        "INSERT INTO event_registrations (id, event_id, user_id)
         VALUES (uuid_generate_v4(), $1, $2)",
        dto.event_id, claims.sub
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Successfully registered for event" })))
}

pub async fn my_membership(
    claims: Claims,
    State(state): State<AppState>,
) -> IndigoResult<Json<Membership>> {
    sqlx::query_as!(
        Membership,
        r#"SELECT id, user_id,
                  tier::text as "tier!",
                  status::text as "status!",
                  stripe_subscription_id,
                  current_period_end, created_at
           FROM memberships WHERE user_id = $1"#,
        claims.sub
    )
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| IndigoError::NotFound("Membership".into()))
}