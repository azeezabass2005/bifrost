use axum::{Json, http::{StatusCode}, response::{IntoResponse, Response}};

use crate::models::Message;

/// The enum for the different kind of error the search endpoint could encounter
#[allow(unused)]
pub enum SearchApiError {
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