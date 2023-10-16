use crate::data_objects::response;
use crate::WebResult;
use serde_json::json;
use std::str;

use vercel_runtime as Vercel;
use warp::Filter;

pub async fn handler() -> response::R {
    const MESSAGE: &str = "Server up and running";

    let result = json!({
        "status": "success".to_string(),
        "message": MESSAGE.to_string(),
    });

    return response::ok(result);
}

pub async fn handler_to_warp() -> WebResult<impl warp::Reply> {
    let result = handler().await;
    return Ok(response::to_warp(result));
}

pub async fn handler_to_vercel() -> Result<Vercel::Response<Vercel::Body>, Vercel::Error> {
    let result = handler().await;

    return Ok(Vercel::Response::builder()
        .status(Vercel::StatusCode::OK)
        .header("content-type", "application/json")
        .body(result.message.to_string().into())?);
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "health")
        .and(warp::get())
        .and_then(handler_to_warp)
}
