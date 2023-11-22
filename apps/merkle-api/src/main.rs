use futures::stream::{StreamExt, TryStreamExt};
use warp::{multipart::FormData, Rejection};

pub mod controller;
mod csv_campaign_parser;
mod data_objects;
mod services;
mod utils;

type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let routes = controller::build_routes();

    // Log a console message only if we are in development mode
    if cfg!(debug_assertions) {
        println!("Compiled successfully!\n");
        println!("You can now interact with the Merkle API web server on:\n");
        println!("http://localhost:8000");
    }

    // Run a web server on localhost:8000
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
