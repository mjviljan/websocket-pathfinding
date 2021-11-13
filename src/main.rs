use warp::{Filter, Rejection, Reply};
use warp::ws::WebSocket;

async fn client_connection(ws: WebSocket) {
    println!("establishing client connection... {:?}", ws);
}

async fn ws_handler(ws: warp::ws::Ws) -> Result<impl Reply, Rejection> {
    println!("ws_handler");
    Ok(ws.on_upgrade(move |socket| client_connection(socket)))
}

#[tokio::main]
async fn main() {
    let hello = warp::path("ws")
        .and(warp::ws())
        .and_then(ws_handler);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
