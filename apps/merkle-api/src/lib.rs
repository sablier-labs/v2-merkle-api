use futures::stream::{StreamExt, TryStreamExt};
use warp::{multipart::FormData, Rejection};

pub mod controller;
pub mod csv_campaign_parser;
pub mod data_objects;
pub mod services;
pub mod utils;

type WebResult<T> = std::result::Result<T, Rejection>;
