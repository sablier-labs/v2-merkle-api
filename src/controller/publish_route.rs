use crate::{
    data_objects::response::{BadRequestResponse, PublishSuccessResponse},
    repository::get_publish_information,
    services::{
        db::with_db,
        ipfs::{try_deserialize_pinata_response, upload_to_ipfs},
    },
    WebResult,
};
use sea_orm::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::json, Filter, Reply};

async fn publish_campaign_handler(gid: String, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();
    let campaign_info = get_publish_information(gid, &db_conn).await;
    match campaign_info {
        Ok(campaign_info) => match campaign_info {
            Some(campaign_info) => {
                let ipfs_response = upload_to_ipfs(campaign_info).await;
                match ipfs_response {
                    Ok(ipfs_response) => {
                        let deserialized_response = try_deserialize_pinata_response(&ipfs_response);
                        match deserialized_response {
                            Ok(deserialized_response) => {
                                let response_json = &PublishSuccessResponse {
                                    status: "Campaign successfully uploaded to IPFS".to_string(),
                                    cid: deserialized_response.ipfs_hash,
                                };
                                return Ok(warp::reply::with_status(
                                    json(response_json),
                                    warp::http::StatusCode::OK,
                                ));
                            }
                            Err(_) => {
                                let response_json = &BadRequestResponse {
                                    message: "There was an error uploading the campaign to ipfs"
                                        .to_string(),
                                };
                                return Ok(warp::reply::with_status(
                                    json(response_json),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                ));
                            }
                        }
                    }
                    Err(_) => {
                        let response_json = &BadRequestResponse {
                            message: "There was an error uploading the campaign to ipfs"
                                .to_string(),
                        };
                        return Ok(warp::reply::with_status(
                            json(response_json),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ));
                    }
                }
            }
            None => {
                let response_json = &BadRequestResponse {
                    message: "Could not find a campaign with the specified gid".to_string(),
                };
                return Ok(warp::reply::with_status(
                    json(response_json),
                    warp::http::StatusCode::BAD_REQUEST,
                ));
            }
        },
        Err(_) => {
            let response_json = &BadRequestResponse {
                message: "There was a problem processing your request.".to_string(),
            };
            return Ok(warp::reply::with_status(
                json(response_json),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
}

pub fn build_route(
    db: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "publish" / String)
        .and(warp::post())
        .and(with_db(db))
        .and_then(publish_campaign_handler)
}
