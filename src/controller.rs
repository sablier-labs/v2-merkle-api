use sea_orm::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::Method, Filter};

pub mod campaign;
pub mod eligibility;
pub mod health;
pub mod publish;
pub mod recipients;
pub mod upload;

pub fn build_routes(
    db_pool: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origins(vec!["http://localhost:3000/", "http://localhost:8000/"])
        .allow_headers(vec!["content-type"])
        .allow_credentials(true);

    let health = health::build_route();
    let upload = upload::build_route(db_pool.clone());
    let campaign = campaign::build_route(db_pool.clone());
    let recipient = recipients::build_route(db_pool.clone());
    let publish = publish::build_route(db_pool.clone());
    let eligibility = eligibility::build_route();

    health
        .with(cors)
        .with(warp::log("api"))
        .or(upload)
        .or(recipient)
        .or(campaign)
        .or(publish)
        .or(eligibility)
}
