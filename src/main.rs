use std::collections::HashMap;

use warp::Filter;

const CLIENT_FOLDER: &str = "client/";

async fn register_new_user(form_data: HashMap<String, String>) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Got registration request...");
    println!("Got body: {:?}", form_data);
    Ok(warp::reply::with_status(format!("Registration done for {}", form_data.get("username").unwrap()), warp::http::StatusCode::OK))
}

#[tokio::main]
async fn main() {
    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::form::<HashMap<String, String>>())
        .and_then(register_new_user);

    let api = register;

    let pages = warp::fs::dir(CLIENT_FOLDER);
    let static_pages = pages;

    let routes = api.or(static_pages);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
