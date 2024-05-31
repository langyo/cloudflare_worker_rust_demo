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

    let val = env
        .kv("axumtest")?
        .get("abc")
        .text()
        .await?
        .map(|v| v.parse().unwrap_or(0))
        .unwrap_or(0);
    let ret = format!("ljyys! Request {} times", val);

    let val = val + 1;
    env.kv("axumtest")?
        .put("abc", val.to_string())?
        .execute()
        .await?;

    Ok(axum::http::Response::new(ret.into()))
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
