use futures::stream::{StreamExt, TryStreamExt};
use warp::{multipart::FormData, Rejection};

mod controller;
mod data_objects;
mod repository;
mod services;
mod utils;
mod database;

type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let db_pool = services::db::establish_connection()
        .await
        .expect("Failed to create db pool");

    let routes = controller::build_routes(db_pool);

    println!("🚀 Server started successfully");
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
