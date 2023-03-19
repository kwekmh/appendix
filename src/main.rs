use std::cell::RefCell;
use std::fmt::Debug;
use appendix::log::*;
use std::str;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
    extract::{Path, State, Json},
};
use axum_macros::debug_handler;

#[derive(Clone)]
struct AppState {
    log: Box<Log>
}

#[derive(Deserialize)]
struct CreateLogParams {
    data: String
}

impl Default for CreateLogParams {
    fn default() -> Self {
        Self { data : "".to_string() }
    }
}

#[tokio::main]
async fn main() {
    // Initialise tracing
    tracing_subscriber::fmt::init();

    // Initialise shared state
    let shared_state = AppState {log: Box::new(Log::new())};

    // Setup routing
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /log` goes to `create_log`
        .route("/log", post(create_log))
        // `GET /log/:id` goes to `read_log
        .route("/log/:id", get(read_log))
        .with_state(shared_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[debug_handler]
async fn create_log(
    State(state): State<AppState>,
    data: Option<Json<CreateLogParams>>
) -> Result<Json<Value>, (StatusCode, String)> {
    match data {
        Some(data) => {
            // let app_state = state.borrow_mut();
            let mut log = *state.log;
            let offset = log.append(Record::new(data.data.as_bytes().to_vec()));
            Ok(Json(json!({ "offset": offset })))
        },
        None => {
            Err((StatusCode::BAD_REQUEST, "Empty data".to_string()))
        }
    }
}

async fn read_log(
    State(state): State<AppState>,
    Path(id): Path<usize>,
) -> (StatusCode, String) {
    let log = *state.log;
    match log.read(id) {
        Ok(v) => {
            let s = match str::from_utf8(&v.value()) {
                Ok(v) => (StatusCode::OK, v.to_string()),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            };
            s
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}
