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

#[tokio::test]
async fn test_register_new_user_with_username_should_succeed() {
    let mut req_data = HashMap::new();
    req_data.insert(String::from("username"), String::from("Whatever"));
    let reg_result = register_new_user(req_data).await;

    assert!(matches!(reg_result, Ok(_)));
}

#[tokio::test]
async fn test_register_new_user_without_username_should_fail() {
    let reg_result = register_new_user(HashMap::new()).await;

    match reg_result {
        Ok(_) => panic!("Registration should fail if no username was given"),
        Err(rejection) => assert!(matches!(rejection.find(), Some(ApiError::UsernameMissing)))
    }
}
