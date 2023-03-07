use axum::{Router, Server, routing::get, extract::State};
use std::{net::SocketAddr, sync::{Mutex, Arc}};

use sysinfo::{CpuExt, System, SystemExt};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(handler))
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

async fn handler(State(state): State<AppState>) -> String {
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