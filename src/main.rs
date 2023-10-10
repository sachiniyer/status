mod sites;
use axum::{extract::ws::WebSocketUpgrade, response::Response, routing::get, Router};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    check_vars();
    let var_map: HashMap<String, String> = env::vars().collect();
    println!("Starting server on {}", var_map.get("BIND_ADDR").unwrap());
    let app = Router::new()
        .route("/", get(sites::handle_http))
        .route("/ws", get(handler));

    axum::Server::bind(&var_map.get("BIND_ADDR").unwrap().parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(sites::handle_ws)
}

fn check_vars() {
    let var_map: HashMap<String, String> = env::vars().collect();
    let var_look = Vec::from(["NGINX", "BIND_ADDR"]);
    let mut res = Vec::new();
    for v in var_look {
        let sites = var_map.get(v);
        match sites {
            Some(_) => {}
            None => res.push(v),
        }
    }
    if !res.is_empty() {
        panic!("{:?} environment variables not set", res);
    }
}
