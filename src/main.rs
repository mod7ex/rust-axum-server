use axum::{
        Router,
        Server,
        routing::get,
        extract::{ State, ws::{WebSocket, Message}, WebSocketUpgrade },
        response::{ IntoResponse, Html },
        http::Response
};
use std::net::SocketAddr;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast::{ channel, Sender };

type SnapShot = Vec<f32>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (tx, _) = channel::<SnapShot>(1);

    let app_state = AppState {
        tx: tx.clone()
    };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/assets/js/index.mjs", get(get_index_js))
        .route("/assets/css/style.css", get(get_index_css))
        /* .route("/cpu-usage", get(preview_cpu_usage)) */
        /* .route("/cpu-usage/json", get(json_preview_cpu_usage)) */
        .route("/realtime", get(realtime_preview_cpu_usage))
        .with_state(app_state);

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let v: Vec<f32> = sys.cpus().iter().map(|cpu| {cpu.cpu_usage()}).collect();
            let _ = tx.send(v);
            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL * 5);
            println!("CPU-usage updated");
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let server = Server::bind(&addr)
        .serve(router.into_make_service());

    println!("Listening on http://{addr}");

    server.await.unwrap();
}

#[derive(Clone)]
struct AppState {
    tx: Sender<SnapShot>,
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();
    /* Html(include_str!("index.html")) */
    Html(markup)
}

#[axum::debug_handler]
async fn get_index_js() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.mjs").await.unwrap();
    
    Response::builder()
        .header("Content-Type", "application/javascript;charset=utf-8")
        .body(markup)
        .unwrap()
}

#[axum::debug_handler]
async fn get_index_css() -> impl IntoResponse {
    let styles = tokio::fs::read_to_string("src/style.css").await.unwrap();
    
    Response::builder()
        .header("Content-Type", "text/css;charset=utf-8")
        .body(styles)
        .unwrap()
}

/*
#[axum::debug_handler]
async fn preview_cpu_usage(State(state): State<AppState>) -> String {
    use std::fmt::Write;

    let mut output = String::new();

    for (i, u) in state.cpu.lock().unwrap().iter().enumerate() {
        let v = u.to_string();
        write!(output, "\n [{i}]: {v}%").unwrap();
    }

    output
}

#[axum::debug_handler]
async fn json_preview_cpu_usage(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.cpu.lock().unwrap().clone())
}
*/

#[axum::debug_handler]
async fn realtime_preview_cpu_usage(
    ws: WebSocketUpgrade,
    State(state): State<AppState>
) -> impl IntoResponse {
    ws.on_upgrade(|ws| async {
        realtime_cpu_stream(state, ws).await 
    })
}

async fn realtime_cpu_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        let payload = serde_json::to_string(&msg).unwrap();
        ws.send(Message::Text(payload)).await.unwrap();
    }
}