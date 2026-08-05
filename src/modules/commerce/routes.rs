use axum::Router;
use axum::routing::{get, post, delete, put};
use crate::state::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/products",
            get(list_products).post(create_product))
        .route("/products/:id",
            get(get_product)
            .put(update_product)
            .delete(delete_product))
        .route("/cart",
            get(get_cart).post(add_to_cart))
        .route("/cart/:product_id",
            delete(remove_from_cart))
        .route("/orders",
            get(my_orders))
        .with_state(state)
}