use axum::{extract::{State, Path}, Json};
use uuid::Uuid;
use crate::{
    AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
    utils::slug::unique_slug,
};
use super::models::*;

pub async fn list_services(
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<BlockchainService>>> {
    let rows = sqlx::query_as!(
        BlockchainService,
        r#"SELECT id, title, slug, description,
                  network::text as "network!",
                  project_type::text as "project_type!",
                  price_from_usd::float8, is_active,
                  created_at, updated_at
           FROM blockchain_services
           WHERE is_active = true
           ORDER BY created_at DESC"#
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get_service(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<BlockchainService>> {
    sqlx::query_as!(
        BlockchainService,
        r#"SELECT id, title, slug, description,
                  network::text as "network!",
                  project_type::text as "project_type!",
                  price_from_usd::float8, is_active,
                  created_at, updated_at
           FROM blockchain_services WHERE slug = $1"#,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| IndigoError::NotFound("Blockchain service".into()))
}

pub async fn submit_inquiry(
    State(state): State<AppState>,
    Json(dto): Json<CreateInquiryDto>,
) -> IndigoResult<Json<serde_json::Value>> {
    sqlx::query!(
        r#"INSERT INTO blockchain_inquiries
              (id, name, email, company, network, project_type, description, budget_range)
           VALUES (uuid_generate_v4(),$1,$2,$3,
                   $4::blockchain_network,$5::blockchain_project_type,$6,$7)"#,
        dto.name, dto.email, dto.company,
        dto.network.as_deref(),
        dto.project_type.as_deref(),
        dto.description, dto.budget_range
    )
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({
        "message": "Inquiry received. We will be in touch within 24 hours."
    })))
}

pub async fn create_project(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreateProjectDto>,
) -> IndigoResult<Json<BlockchainProject>> {
    let id = Uuid::new_v4();
    let row = sqlx::query_as!(
        BlockchainProject,
        r#"INSERT INTO blockchain_projects
              (id, client_id, title, description, network, project_type, budget_usd)
           VALUES ($1,$2,$3,$4,$5::blockchain_network,$6::blockchain_project_type,$7)
           RETURNING id, client_id, title, description,
                     network::text as "network!",
                     project_type::text as "project_type!",
                     status::text as "status!",
                     budget_usd::float8, created_at, updated_at"#,
        id, claims.sub, dto.title, dto.description,
        dto.network, dto.project_type, dto.budget_usd
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn my_projects(
    claims: Claims,
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<BlockchainProject>>> {
    let rows = sqlx::query_as!(
        BlockchainProject,
        r#"SELECT id, client_id, title, description,
                  network::text as "network!",
                  project_type::text as "project_type!",
                  status::text as "status!",
                  budget_usd::float8, created_at, updated_at
           FROM blockchain_projects
           WHERE client_id = $1
           ORDER BY created_at DESC"#,
        claims.sub
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}