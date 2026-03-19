use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};

/// The structure of the search request body
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRequestBody {
    pub product_name: String,
    pub sites: Option<Vec<String>>,
    pub location: Option<String>
}

/// The structure of the search response body
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponseBody {
    // TODO: I will add necessary fields to the search response body later
    message: String
}

impl SearchResponseBody {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl IntoResponse for SearchResponseBody {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub message: String
}