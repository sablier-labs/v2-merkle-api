use futures::stream::{StreamExt, TryStreamExt};
use sea_orm::DbConn;
use warp::{http::Method, multipart::FormData, Filter, Rejection};
use std::sync::Arc;
use tokio::sync::Mutex;

mod handler;
mod response;
mod utils;
mod connect;
mod entities;

type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let db_pool = connect::establish_connection().await.expect("Failed to create db pool");

    let health_checker = warp::path!("api" / "healthchecker")
        .and(warp::get())
        .and_then(handler::health_checker_handler);

    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origins(vec!["http://localhost:3000/", "http://localhost:8000/"])
        .allow_headers(vec!["content-type"])
        .allow_credentials(true);

    let upload_route = warp::path!("api" / "upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000))
        .and(with_db(db_pool.clone()))
        .and_then(handler::upload_handler);

    let routes = health_checker
        .with(cors)
        .with(warp::log("api"))
        .or(upload_route);

    println!("🚀 Server started successfully");
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}

fn with_db(db_pool: Arc<Mutex<DbConn>>) -> impl Filter<Extract = (Arc<Mutex<DbConn>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}



