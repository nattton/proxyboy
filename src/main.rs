use dotenvy::dotenv;
use std::{env, fs};

use axum::{
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::{delete, get, patch, put},
    Router,
};
use serde::Deserialize;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route(
            "/*path",
            get(get_handler)
                .post(post_handler)
                .patch(patch(patch_handler))
                .put(put(put_handler))
                .delete(delete(delete_handler)),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    dotenv().ok();
    let server_addr = env::var("SERVER_ADDR").unwrap_or("0.0.0.0:3000".to_string());
    println!("Running server on http://{}", server_addr);
    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn response(method: &str, path: String) -> Result<String, String> {
    let contents =
        fs::read_to_string("config.json").expect("Should have been able to read the file");
    let config: Config = serde_json::from_str(&contents).expect("JSON was not well-formatted");
    for router in config.router_list {
        if router.method.to_lowercase().contains(method) && router.url.trim().contains(&path) {
            println!("{}\t{}\t{}", method, router.url, router.file);
            let response = fs::read_to_string(format!("{}{}", config.store_path, router.file))
                .unwrap_or("not found".to_string());
            return Ok(response);
        }
    }
    Err("Not Found".to_string())
}

fn print_path(path: &str) {
    println!("### Path ###\t{}", path);
}
fn print_headers(headers: HeaderMap) {
    println!("### Headers ###");
    for header in headers.iter() {
        println!("{}\t\t{:?}", header.0, header.1);
    }
    println!("### End Headers ###");
}

async fn get_handler(Path(path): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    print_path(&path);
    print_headers(headers);
    match response("get", path) {
        Ok(response) => ([(header::CONTENT_TYPE, "application/json")], response),
        Err(err) => ([(header::CONTENT_TYPE, "text/plain")], err),
    }
}

async fn post_handler(Path(path): Path<String>) -> impl IntoResponse {
    match response("post", path) {
        Ok(response) => ([(header::CONTENT_TYPE, "application/json")], response),
        Err(err) => ([(header::CONTENT_TYPE, "text/plain")], err),
    }
}

async fn patch_handler(Path(path): Path<String>) -> impl IntoResponse {
    match response("patch", path) {
        Ok(response) => ([(header::CONTENT_TYPE, "application/json")], response),
        Err(err) => ([(header::CONTENT_TYPE, "text/plain")], err),
    }
}

async fn put_handler(Path(path): Path<String>) -> impl IntoResponse {
    match response("patch", path) {
        Ok(response) => ([(header::CONTENT_TYPE, "application/json")], response),
        Err(err) => ([(header::CONTENT_TYPE, "text/plain")], err),
    }
}

async fn delete_handler(Path(path): Path<String>) -> impl IntoResponse {
    match response("delete", path) {
        Ok(response) => ([(header::CONTENT_TYPE, "application/json")], response),
        Err(err) => ([(header::CONTENT_TYPE, "text/plain")], err),
    }
}

#[derive(Deserialize)]
struct Config {
    store_path: String,
    router_list: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    method: String,
    url: String,
    file: String,
}
