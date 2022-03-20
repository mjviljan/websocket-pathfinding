use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;

use warp::{Filter, reject, Rejection, reply, Reply};
use warp::http::StatusCode;

const CLIENT_FOLDER: &str = "client/";
const DATA_KEY_USERNAME: &str = "username";

#[derive(Debug)]
enum ApiError {
    UsernameMissing
}

impl reject::Reject for ApiError {}

async fn register_new_user(form_data: HashMap<String, String>, existing_users: Arc<Mutex<HashSet<String>>>) -> Result<impl Reply, Rejection> {
    println!("Got registration request...");
    match form_data.get(DATA_KEY_USERNAME) {
        Some(username) => {
            let mut users = existing_users.lock().await;
            if users.contains(username) {
                Ok(reply::with_status("ERROR: User already exists", StatusCode::BAD_REQUEST))
            } else {
                users.insert(username.clone());
                Ok(reply::with_status("Registration done", StatusCode::OK))
            }
        }
        None => Err(reject::custom(ApiError::UsernameMissing))
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(ApiError::UsernameMissing) = err.find() {
        println!("Handling error: {:?}", err);
        Ok(reply::with_status("ERROR: No username was given", StatusCode::BAD_REQUEST))
    } else {
        Err(err)
    }
}

fn with_users(users: Arc<Mutex<HashSet<String>>>) -> impl Filter<Extract=(Arc<Mutex<HashSet<String>>>, ), Error=Infallible> + Clone {
    warp::any().map(move || users.clone())
}

#[tokio::main]
async fn main() {
    let users = Arc::new(Mutex::new(HashSet::<String>::new()));

    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::content_length_limit(128))
        .and(warp::body::form())
        .and(with_users(users.clone()))
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
    let req_data = HashMap::from([(String::from(DATA_KEY_USERNAME), String::from("Whatever"))]);
    let reg_result = register_new_user(req_data, Arc::new(Mutex::new(HashSet::new()))).await;

    assert!(matches!(reg_result, Ok(_)));
}

#[tokio::test]
async fn test_register_new_user_without_username_should_fail() {
    let reg_result = register_new_user(HashMap::new(), Arc::new(Mutex::new(HashSet::new()))).await;

    match reg_result {
        Ok(_) => panic!("Registration should fail if no username was given"),
        Err(rejection) => assert!(matches!(rejection.find(), Some(ApiError::UsernameMissing)))
    }
}

#[tokio::test]
async fn test_register_new_user_with_existing_username_should_fail() {
    let existing_users = Arc::new(Mutex::new(HashSet::<String>::new()));

    let req_data = HashMap::from([(String::from(DATA_KEY_USERNAME), String::from("NewUser"))]);

    let _first_reg_result = register_new_user(req_data.clone(), existing_users.clone()).await;
    let second_reg_result = register_new_user(req_data.clone(), existing_users).await;

    match second_reg_result {
        Ok(msg) => assert_eq!(msg.into_response().status(), StatusCode::BAD_REQUEST),
        Err(_) => panic!("Trying to register a user with an already taken name should return an OK result with a message")
    }
}
