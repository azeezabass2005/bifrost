use axum::{Router, routing::post};
use crate::api::execute_search;

pub mod models;
pub mod error;
pub mod api;


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/search", post(execute_search));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    println!("Listening on: {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

