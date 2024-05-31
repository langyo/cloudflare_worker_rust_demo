use axum::{routing::get, Router};
use kv::ToRawKvValue;
use tower_service::Service;
use worker::*;

fn router() -> Router {
    Router::new().route("/", get(root))
}

// https://developers.cloudflare.com/workers/languages/rust/
#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    // Ok(router().call(req).await?)

    Ok(axum::http::Response::new(
        env.kv("axumtest")?
            .get("abc")
            .text()
            .await?
            .unwrap_or("No key found".to_string())
            .into(),
    ))
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
