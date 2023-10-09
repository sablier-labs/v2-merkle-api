use futures::stream::{StreamExt, TryStreamExt};
use warp::{multipart::FormData, Rejection};

mod controller;
mod csv_campaign_parser;
mod data_objects;
mod services;
mod utils;

type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let routes = controller::build_routes();

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
