use sea_orm::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{http::Method, Filter};

pub mod campaign_route;
pub mod eligibility_route;
pub mod health_route;
pub mod publish_route;
pub mod recipients_route;
pub mod upload_route;

pub fn build_routes(
    db_pool: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origins(vec!["http://localhost:3000/", "http://localhost:8000/"])
        .allow_headers(vec!["content-type"])
        .allow_credentials(true);

    let health = health_route::build_route();
    let upload = upload_route::build_route(db_pool.clone());
    let campaign = campaign_route::build_route(db_pool.clone());
    let recipient = recipients_route::build_route(db_pool.clone());
    let publish = publish_route::build_route(db_pool.clone());
    let eligibility = eligibility_route::build_route();

    health
        .with(cors)
        .with(warp::log("api"))
        .or(upload)
        .or(recipient)
        .or(campaign)
        .or(publish)
        .or(eligibility)
}
