mod sites;
use axum::{extract::ws::WebSocketUpgrade, response::Response, routing::get, Router};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    check_vars();

    let app = Router::new()
        .route("/", get(sites::handle_http))
        .route("/ws", get(handler));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(sites::handle_ws)
}

fn check_vars() {
    let var_map: HashMap<String, String> = env::vars().collect();
    let sites = var_map.get("NGINX");
    match sites {
        Some(_) => {}
        None => {
            panic!("NGINX environment variable not set");
        }
    }
}
