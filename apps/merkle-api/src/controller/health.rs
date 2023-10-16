use crate::data_objects::response;
use crate::{data_objects::response::GenericResponse, WebResult};

use std::str;
use warp::Filter;
use warp::{reply::json, Reply};

pub async fn handler() -> GenericResponse {
    const MESSAGE: &str = "Server up and running";

    let response_json = GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };

    response_json
}

pub async fn build_target_warp() -> WebResult<impl Reply> {
    let response = handler().await;
    Ok(response::ok(json(&response)))
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "health")
        .and(warp::get())
        .and_then(build_target_warp)
}
