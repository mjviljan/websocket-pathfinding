use std::collections::HashMap;

use warp::{Filter, reject, Rejection, reply, Reply};
use warp::http::StatusCode;

const CLIENT_FOLDER: &str = "client/";

#[derive(Debug)]
enum ApiError {
    UsernameMissing
}

impl reject::Reject for ApiError {}

async fn register_new_user(form_data: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    println!("Got registration request...");
    match form_data.get("username") {
        Some(username) => Ok(reply::with_status(format!("Registration done for {}", username), StatusCode::OK)),
        None => Err(reject::custom(ApiError::UsernameMissing))
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    println!("Handling error: {:?}", err);

    if let Some(ApiError::UsernameMissing) = err.find() {
        Ok(reply::with_status("ERROR: No username was given", StatusCode::BAD_REQUEST))
    } else {
        Err(err)
    }
}

#[tokio::main]
async fn main() {
    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::content_length_limit(128))
        .and(warp::body::form())
        .and_then(register_new_user)
        .recover(handle_rejection);

    let api = register;

    let pages = warp::fs::dir(CLIENT_FOLDER);
    let static_pages = pages;

    let routes = api.or(static_pages);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
