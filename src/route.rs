use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use std::sync::Arc;
use worker::{console_error, Env};

#[derive(Clone)]
struct CFEnv {
    pub env: Arc<Env>,
}

unsafe impl Send for CFEnv {}
unsafe impl Sync for CFEnv {}

pub fn router(env: Env) -> Router {
    let state = CFEnv { env: Arc::new(env) };

    Router::new()
        .route("/", get(test))
        .route("/test", get(handler))
        .with_state(state)
}

async fn test() -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok("ljyys!")
}

async fn handler(State(state): State<CFEnv>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let env = state.env.clone();
    let (tx, rx) = oneshot::channel();

    wasm_bindgen_futures::spawn_local(async move {
        let ret: Result<i32> = {
            let val = env
                .kv("ljyys")
                .expect("Failed to get kv namespace")
                .get("abc")
                .text()
                .await
                .expect("Failed to get value from kv");
            let val = val.map(|val| val.parse().unwrap_or(0)).unwrap_or(0);

            let val = val + 1;
            env.kv("ljyys")
                .expect("Failed to get kv namespace")
                .put("abc", val.to_string())
                .expect("Failed to put value to kv")
                .execute()
                .await
                .expect("Failed to execute put value to kv");

            Ok(val)
        };

        match ret {
            Ok(ret) => {
                tx.send(ret).unwrap();
            }
            Err(err) => {
                console_error!("Error: {:?}", err);
            }
        }
    });

    let ret = rx.await.unwrap();
    let ret = format!("ljyys! Request {} times", ret);
    Ok(ret.into_response())
}
