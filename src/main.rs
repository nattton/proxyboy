use diesel::prelude::*;
use dotenvy::dotenv;
use proxyboy::{establish_connection, import, schema::mocks::dsl::*};
use std::{env, fmt::format, fs, thread::sleep, time::Duration};

use axum::{
    extract::Path,
    http::{header, HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::any,
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use proxyboy::models::Mock;

// Add custom error type
#[derive(Debug)]
enum ApiError {
    FileReadError(std::io::Error),
    RouteNotFound,
}

// Implement response conversion for ApiError
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
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
    // Check for import command
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "import" {
        println!("Importing config.json to database...");
        match import::import_config() {
            Ok(()) => println!("Import completed successfully!"),
            Err(e) => {
                eprintln!("Import failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
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
    let path_url = path.strip_prefix('/').unwrap_or(&path);

    let mut conn = establish_connection();
    let mock = mocks
        .filter(is_enable.eq(true))
        .filter(
            request_method
                .like(format!("%{}%", method))
                .or(request_method.eq("*")),
        )
        .filter(request_url.like(format!("%{}", path_url)))
        .first::<Mock>(&mut conn)
        .optional()
        .expect("Error loading mocks");

    match mock {
        Some(mock) => {
            let store_path = env::var("STORE_PATH").unwrap_or("store".to_string());

            let delay = mock.response_delay;

            let file = format!("{}{}", store_path, mock.response_file_path);

            println!("{}\t{}\t{}\t{}", method, delay, path_url, file);

            let body = fs::read_to_string(&file).map_err(ApiError::FileReadError)?;

            if delay > 0 {
                sleep(Duration::from_millis(delay as u64));
            }

            return Ok(Response {
                status_code: StatusCode::from_u16(mock.response_status_code as u16)
                    .unwrap_or(StatusCode::OK),
                content_type: mock.response_content_type,
                body,
            });
        }
        None => {
            return Err(ApiError::RouteNotFound);
        }
    }
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

struct Response {
    status_code: StatusCode,
    content_type: String,
    body: String,
}
