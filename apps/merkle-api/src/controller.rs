use warp::{http::Method, Filter};

pub mod create;
pub mod eligibility;
pub mod health;

pub fn build_routes(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origins(vec!["http://localhost:3000/", "http://localhost:8000/"])
        .allow_headers(vec!["content-type"])
        .allow_credentials(true);

    let health = health::build_route();
    let create = create::build_route();
    let eligibility = eligibility::build_route();

    health
        .with(cors)
        .with(warp::log("api"))
        .or(eligibility)
        .or(create)
}
