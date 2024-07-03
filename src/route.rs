use anyhow::Result;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use worker::{console_error, Env};

use sea_orm::EntityTrait;

#[derive(Clone)]
struct CFEnv {
    pub env: Arc<Env>,
}

unsafe impl Send for CFEnv {}
unsafe impl Sync for CFEnv {}

pub fn router(env: Env) -> Router {
    let state = CFEnv { env: Arc::new(env) };

    Router::new().route("/", get(handler)).with_state(state)
}

async fn handler(State(state): State<CFEnv>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let env = state.env.clone();
    let db = crate::orm::init_db(env).await.map_err(|err| {
        console_error!("Failed to connect to database: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to connect to database".to_string(),
        )
    })?;

    let ret = crate::entity::Entity::find()
        .all(&db)
        .await
        .map_err(|err| {
            console_error!("Failed to query database: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to query database".to_string(),
            )
        })?;
    let ret = serde_json::to_string(&ret).map_err(|err| {
        console_error!("Failed to serialize response: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize response".to_string(),
        )
    })?;

    Ok(ret.into_response())
}
