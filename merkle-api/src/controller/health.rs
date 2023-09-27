use crate::data_objects::response;
use crate::{data_objects::response::GenericResponse, WebResult};

use std::str;
use warp::Filter;
use warp::{reply::json, Reply};

async fn health_checker_handler() -> WebResult<impl Reply> {
    const MESSAGE: &str = "Server up and running";

    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };

    return Ok(response::ok(json(response_json)));
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "healthchecker")
        .and(warp::get())
        .and_then(health_checker_handler)
}
