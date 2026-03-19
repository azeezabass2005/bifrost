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

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub message: String
}

/// A single product listing
#[derive(Serialize, Deserialize, Debug)]
pub struct ListingResult {
    vendor: Option<String>,
    link: String,
    description: String,
    good_reviews: Vec<String>,
    bad_reviews: Vec<String>,
    price: String,
}

/// The Centralized/Generic structure for all site results
#[derive(Serialize, Deserialize, Debug)]
pub struct SiteResult {
    results: Vec<ListingResult>
}

/// The Centralized/Generic structure for any error encountered on all site
#[derive(Serialize, Deserialize, Debug)]
pub enum SiteError {
    ProductNotFound,
    ScrapingNotAllowed,
    SiteNotAvailable
}