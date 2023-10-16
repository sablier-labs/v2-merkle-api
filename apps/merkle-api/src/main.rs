use dotenvy::dotenv;
use futures::stream::{StreamExt, TryStreamExt};
use warp::{multipart::FormData, Filter, Rejection};

mod controller;
mod csv_campaign_parser;
mod data_objects;
mod services;
mod utils;

type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    // dotenv().ok();

    // if let Ok(mode) = std::env::var("RUN_MODE") {
    //     if mode == "LAMBDA" {
    //         let routes = warp::any().map(|| "Hello, World!");
    //         let warp_service = warp::service(routes);
    //         warp_lambda::run(warp_service)
    //             .await
    //             .expect("An error occured");
    //         return;
    //     }
    // }

    // let routes = controller::build_routes();
    // warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;

    let routes = warp::any().map(|| "Hello, World!");
    let warp_service = warp::service(routes);
    warp_lambda::run(warp_service)
        .await
        .expect("An error occured");
}
