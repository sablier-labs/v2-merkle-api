use warp::{http::Method, Filter};

pub mod create;
pub mod eligibility;
pub mod health;
pub mod validity;

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, std::convert::Infallible> {
    Ok(warp::reply::json(&format!("{:?}", err)))
}

pub fn build_routes() -> impl warp::Filter<Extract = impl warp::Reply> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_any_origin()
        .allow_headers(vec!["Origin", "Content-Type", "X-Auth-Token", "X-AppId"]);

    let health = health::build_route();
    let create = create::build_route();
    let eligibility = eligibility::build_route();
    let validity = validity::build_route();

    health.or(eligibility).or(create).or(validity).recover(handle_rejection).with(cors).with(warp::log("api"))
}
