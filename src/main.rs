use axum::{Router, Server, routing::get};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let server = Server::bind(&addr)
        .serve(router.into_make_service());

    println!("Listening on http://{addr}");

    server.await.unwrap();
}

async fn handler() -> &'static str {
    "Hi bro"
}