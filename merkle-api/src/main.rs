use futures::stream::{StreamExt, TryStreamExt};
use warp::{multipart::FormData, Rejection};

mod controller;
mod data_objects;
mod repository;
mod services;
mod utils;
mod database;
mod csv_campaign_parser;

type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let db_pool = database::management::establish_connection()
        .await
        .expect("Failed to create db pool");

    let routes = controller::build_routes(db_pool);

    println!("🚀 Server started successfully");
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
