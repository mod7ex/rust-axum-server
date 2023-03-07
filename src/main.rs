use axum::{Router, Server, routing::get, extract::State, Json, response::IntoResponse};
use std::{net::SocketAddr, sync::{Mutex, Arc}};

use sysinfo::{CpuExt, System, SystemExt};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(root_get))
        .route("/cpu-usage", get(preview_cpu_usage))
        .route("/cpu-usage/json", get(json_preview_cpu_usage))
        .with_state(AppState {
            sys: Arc::new(Mutex::new(System::new()))
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let server = Server::bind(&addr)
        .serve(router.into_make_service());

    println!("Listening on http://{addr}");

    server.await.unwrap();
}

#[derive(Clone)]
struct AppState {
    sys: Arc<Mutex<System>>
}

#[axum::debug_handler]
async fn root_get() -> &'static str {
    "Hello"
}

#[axum::debug_handler]
async fn preview_cpu_usage(State(state): State<AppState>) -> String {
    use std::fmt::Write;

    let mut sys = state.sys.lock().unwrap();

    sys.refresh_cpu();

    let mut output = String::new();

    for cpu in sys.cpus() {
        let n = cpu.name();
        let v = cpu.cpu_usage().to_string();
        write!(output, "\n [{n}]: {v}%").unwrap();
    }

    output
}

#[axum::debug_handler]
async fn json_preview_cpu_usage(State(state): State<AppState>) -> impl IntoResponse {
    let mut sys = state.sys.lock().unwrap();

    sys.refresh_cpu();

    let v: Vec<f32> = sys.cpus()
        .iter()
        .map(|cpu| {
            cpu.cpu_usage()
        })
        .collect();

    Json(v)
}