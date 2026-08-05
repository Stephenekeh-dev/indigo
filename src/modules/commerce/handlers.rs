use axum::{extract::{State, Path}, Json};
use uuid::Uuid;
use crate::{
    state::AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
    utils::slug::unique_slug,
};
use super::models::*;

pub async fn list_products(
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<Product>>> {
    let rows = sqlx::query_as!(
        Product,
        r#"SELECT id, title, slug, description, short_desc,
                  product_type::text as "product_type!",
                  status::text as "status!",
                  price_usd::float8 as "price_usd!",
                  compare_price::float8 as compare_price,
                  is_digital, download_url, thumbnail_url,
                  tags, stock_count, created_at, updated_at
           FROM products
           WHERE status = 'active'
           ORDER BY sort_order ASC"#
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<Product>> {
    sqlx::query_as!(
        Product,
        r#"SELECT id, title, slug, description, short_desc,
                  product_type::text as "product_type!",
                  status::text as "status!",
                  price_usd::float8 as "price_usd!",
                  compare_price::float8 as compare_price,
                  is_digital, download_url, thumbnail_url,
                  tags, stock_count, created_at, updated_at
           FROM products
           WHERE slug = $1 AND status = 'active'"#,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| IndigoError::NotFound("Product".into()))
}

pub async fn create_product(
    _claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreateProductDto>,
) -> IndigoResult<Json<Product>> {
    let id         = Uuid::new_v4();
    let slug       = unique_slug(&dto.title, &id);
    let tags       = dto.tags.unwrap_or_default();
    let is_digital = dto.is_digital.unwrap_or(true);
    let row = sqlx::query_as!(
        Product,
        r#"INSERT INTO products
              (id, title, slug, description, short_desc,
               product_type, price_usd, is_digital, tags)
           VALUES ($1,$2,$3,$4,$5,$6::text::product_type,$7::float8,$8,$9)
           RETURNING id, title, slug, description, short_desc,
                     product_type::text as "product_type!",
                     status::text as "status!",
                     price_usd::float8 as "price_usd!",
                     compare_price::float8 as compare_price,
                     is_digital, download_url, thumbnail_url,
                     tags, stock_count, created_at, updated_at"#,
        id, dto.title, slug, dto.description, dto.short_desc,
        dto.product_type, dto.price_usd, is_digital, &tags
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn add_to_cart(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<AddToCartDto>,
) -> IndigoResult<Json<serde_json::Value>> {
    let qty = dto.quantity.unwrap_or(1).max(1);
    sqlx::query!(
        r#"INSERT INTO cart_items (id, user_id, product_id, quantity)
           VALUES (uuid_generate_v4(), $1, $2, $3)
           ON CONFLICT (user_id, product_id)
           DO UPDATE SET quantity = cart_items.quantity + EXCLUDED.quantity"#,
        claims.sub, dto.product_id, qty
    )
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "message": "Added to cart" })))
}

pub async fn get_cart(
    claims: Claims,
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<CartItem>>> {
    let rows = sqlx::query_as!(
        CartItem,
        "SELECT id, user_id, product_id, quantity, added_at
         FROM cart_items WHERE user_id = $1",
        claims.sub
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn remove_from_cart(
    claims: Claims,
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> IndigoResult<Json<serde_json::Value>> {
    sqlx::query!(
        "DELETE FROM cart_items WHERE user_id = $1 AND product_id = $2",
        claims.sub, product_id
    )
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "message": "Removed from cart" })))
}

pub async fn my_orders(
    claims: Claims,
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<Order>>> {
    let rows = sqlx::query_as!(
        Order,
        r#"SELECT id, user_id,
                  status::text as "status!",
                  total_usd::float8 as "total_usd!",
                  stripe_payment_id, billing_email,
                  created_at, updated_at
           FROM orders
           WHERE user_id = $1
           ORDER BY created_at DESC"#,
        claims.sub
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn update_product(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<CreateProductDto>,
) -> IndigoResult<Json<Product>> {
    let tags       = dto.tags.unwrap_or_default();
    let is_digital = dto.is_digital.unwrap_or(true);
    let row = sqlx::query_as!(
        Product,
        r#"UPDATE products SET
              title        = $1,
              description  = $2,
              short_desc   = $3,
              product_type = $4::text::product_type,
              price_usd    = $5::float8,
              is_digital   = $6,
              tags         = $7,
              updated_at   = NOW()
           WHERE id = $8
           RETURNING id, title, slug, description, short_desc,
                     product_type::text as "product_type!",
                     status::text as "status!",
                     price_usd::float8 as "price_usd!",
                     compare_price::float8 as compare_price,
                     is_digital, download_url, thumbnail_url,
                     tags, stock_count, created_at, updated_at"#,
        dto.title, dto.description, dto.short_desc,
        dto.product_type, dto.price_usd, is_digital, &tags, id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete_product(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> IndigoResult<Json<serde_json::Value>> {
    sqlx::query!(
        "DELETE FROM products WHERE id = $1", id
    )
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "message": "Product deleted" })))
}