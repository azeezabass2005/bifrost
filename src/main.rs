use axum::{Json, Router, extract::rejection::JsonRejection, http::StatusCode, response::{IntoResponse, Response}, routing::post};
use serde::{Serialize, Deserialize};

/// The structure of the search request body
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRequestBody {
    product_name: String,
    sites: Option<Vec<String>>,
    location: Option<String>
}

/// The structure of the search response body
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponseBody {
    // TODO: I will add necessary fields to the search response body later
    message: String
}

impl SearchResponseBody {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl IntoResponse for SearchResponseBody {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Message {
    message: String
}

/// The enum for the different kind of error the search endpoint could encounter
#[allow(unused)]
enum SearchApiError {
    BadRequest(Message),
    InternalServerError
}

impl IntoResponse for SearchApiError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, Json(message)).into_response(),
            Self::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/search", post(execute_search));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    println!("Listening on: {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// The function that executes the search
async fn execute_search(payload: Result<Json<SearchRequestBody>, JsonRejection>) -> Result<SearchResponseBody, SearchApiError> {
    let Json(body) = payload.map_err(|rej| {
        SearchApiError::BadRequest(Message {
            message: rej.to_string(),
        })
    })?;
    println!("This is the request body: {:?}", body);
    Ok(SearchResponseBody::new(format!("Search working: {}", body.product_name)))
}
