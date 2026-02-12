use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use pocketclaw_core::bus::{Event, MessageBus};
use pocketclaw_core::types::{Message, Role};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    bus: Arc<MessageBus>,
}

pub struct Gateway {
    bus: Arc<MessageBus>,
    port: u16,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
    version: &'static str,
    uptime: &'static str,
}

#[derive(Deserialize)]
struct SendMessageRequest {
    message: String,
    #[serde(default = "default_session_key")]
    session_key: String,
}

fn default_session_key() -> String {
    format!("http:{}", Uuid::new_v4())
}

#[derive(Serialize)]
struct SendMessageResponse {
    id: String,
    status: &'static str,
}

impl Gateway {
    pub fn new(bus: Arc<MessageBus>, port: u16) -> Self {
        Self { bus, port }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let state = AppState {
            bus: self.bus.clone(),
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .route("/api/status", get(api_status))
            .route("/api/message", post(send_message))
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("Gateway listening on {}", addr);

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: "0.1.0",
    })
}

async fn api_status(State(_state): State<AppState>) -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "running",
        version: "0.1.0",
        uptime: "N/A",
    })
}

/// POST /api/message — send a message to the agent via HTTP
async fn send_message(
    State(state): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, StatusCode> {
    let msg_id = Uuid::new_v4();

    let msg = Message {
        id: msg_id,
        channel: "http".to_string(),
        session_key: req.session_key,
        content: req.message,
        role: Role::User,
        metadata: Default::default(),
    };

    state
        .bus
        .publish(Event::InboundMessage(msg))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SendMessageResponse {
        id: msg_id.to_string(),
        status: "accepted",
    }))
}
