use dotenvy::dotenv;
use std::{env, fmt::format, fs, thread::sleep, time::Duration};

use axum::{
    extract::Path,
    http::{header, HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{any, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

// Add custom error type
#[derive(Debug)]
enum ApiError {
    ConfigReadError(std::io::Error),
    ConfigParseError(serde_json::Error),
    FileReadError(std::io::Error),
    RouteNotFound,
}

// Implement response conversion for ApiError
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::ConfigReadError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format(format_args!("Config file error : {}", error.to_string())),
            ),
            ApiError::ConfigParseError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format(format_args!("Config parse error : {}", error.to_string())),
            ),
            ApiError::FileReadError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format(format_args!("Response file error : {}", error.to_string())),
            ),
            ApiError::RouteNotFound => (
                StatusCode::NOT_FOUND,
                format(format_args!("Route not found")),
            ),
        };
        (status, message).into_response()
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/parse_token", post(parse_token))
        .route("/{*path}", any(handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    dotenv().ok();
    let server_addr = env::var("SERVER_ADDR").unwrap_or("0.0.0.0:3000".to_string());
    println!("Running server on http://{}", server_addr);
    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn response(headers: &HeaderMap, method: &str, path: String) -> Result<Response, ApiError> {
    let host = headers.get("host").unwrap().to_str().unwrap_or("");
    println!("### Request ###\t{}\t{}\t{}", method, host, path);

    let contents = fs::read_to_string("config.json").map_err(ApiError::ConfigReadError)?;

    let config: Config = serde_json::from_str(&contents).map_err(ApiError::ConfigParseError)?;

    let path_url = path.strip_prefix('/').unwrap_or(&path);

    for router in config.router_list {
        let enable = router.enable.unwrap_or(true);
        let router_url = router.url.trim().strip_prefix('/').unwrap_or(&router.url);
        if enable && router.method.to_uppercase().contains(method) && router_url.eq(path_url) {
            let delay = router.delay.unwrap_or(0);
            let mode_path = if config.mode.is_empty() {
                ".json".to_string()
            } else {
                format!("_{}.json", config.mode)
            };

            let file = format!(
                "{}{}",
                config.store_path,
                router.file.replace(".json", &mode_path)
            );

            println!(
                "{}\t{}\t{}\t{}\t{}",
                config.mode, method, delay, router.url, file
            );

            let body = fs::read_to_string(&file)
                .or_else(|_| fs::read_to_string(&config.store_path))
                .map_err(ApiError::FileReadError)?;

            if delay > 0 {
                sleep(Duration::from_millis(delay));
            }

            return Ok(Response {
                status_code: StatusCode::from_u16(router.status_code.unwrap_or(200))
                    .unwrap_or(StatusCode::OK),
                content_type: router
                    .content_type
                    .unwrap_or_else(|| "application/json".to_string()),
                body,
            });
        }
    }
    Err(ApiError::RouteNotFound)
}

fn print_request(method: &Method, path: &str, headers: &HeaderMap, body: &str) {
    println!("### Method ###\t{}", method);
    println!("### Path ###\t{}", path);
    println!("### Headers ###");
    for header in headers.iter() {
        println!("{}\t\t{:?}", header.0, header.1);
    }
    println!("### End Headers ###");
    println!("### Body ###\t{}", body);
}

async fn handler(
    method: Method,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    print_request(&method, &path, &headers, &body);

    match response(&headers, method.as_str(), path) {
        Ok(response) => (
            response.status_code,
            [(header::CONTENT_TYPE, response.content_type)],
            response.body,
        )
            .into_response(),
        Err(err) => err.into_response(),
    }
}

async fn parse_token(Json(payload): Json<ParseTokenRequest>) -> (StatusCode, String) {
    let add_token_code = format!(
        "localStorage.setItem('accessToken', '{}')
    localStorage.setItem('refreshToken', '{}')",
        payload.data.access_token, payload.data.refresh_token
    );
    (StatusCode::OK, add_token_code)
}

struct Response {
    status_code: StatusCode,
    content_type: String,
    body: String,
}

#[derive(Deserialize)]
struct Config {
    store_path: String,
    mode: String,
    router_list: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    enable: Option<bool>,
    method: String,
    url: String,
    file: String,
    delay: Option<u64>,
    status_code: Option<u16>,
    content_type: Option<String>,
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct ParseTokenRequest {
    data: Token,
}
