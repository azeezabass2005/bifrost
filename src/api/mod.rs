use axum::{Json, extract::rejection::JsonRejection};

use crate::{error::SearchApiError, models::{Message, SearchRequestBody, SearchResponseBody}};

/// The function that executes the search
pub async fn execute_search(payload: Result<Json<SearchRequestBody>, JsonRejection>) -> Result<SearchResponseBody, SearchApiError> {
    let Json(body) = payload.map_err(|rej| {
        SearchApiError::BadRequest(Message {
            message: rej.to_string(),
        })
    })?;
    println!("This is the request body: {:?}", body);
    Ok(SearchResponseBody::new(format!("Search working: {}", body.product_name)))
}