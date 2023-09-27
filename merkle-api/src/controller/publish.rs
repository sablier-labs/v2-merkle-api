use crate::{
    data_objects::response::{BadRequestResponse, PublishSuccessResponse},
    database::management::with_db,
    repository,
    services::ipfs::{try_deserialize_pinata_response, upload_to_ipfs},
    WebResult,
};
use sea_orm::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::json, Filter, Reply};

async fn publish_campaign_handler(gid: String, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();
    let campaign_info = repository::campaign::get_publish_information(gid, &db_conn).await;

    if let Err(_) = campaign_info {
        let response_json = &BadRequestResponse {
            message: "There was a problem processing your request.".to_string(),
        };
        return Ok(warp::reply::with_status(
            json(response_json),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let campaign_info = campaign_info.unwrap();

    if let None = campaign_info {
        let response_json = &BadRequestResponse {
            message: "Could not find a campaign with the specified gid".to_string(),
        };
        return Ok(warp::reply::with_status(
            json(response_json),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let campaign_info = campaign_info.unwrap();
    let ipfs_response = upload_to_ipfs(campaign_info).await;
    if let Err(_) = ipfs_response {
        let response_json = &BadRequestResponse {
            message: "There was an error uploading the campaign to ipfs".to_string(),
        };
        return Ok(warp::reply::with_status(
            json(response_json),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let ipfs_response = ipfs_response.unwrap();
    let deserialized_response = try_deserialize_pinata_response(&ipfs_response);

    if let Err(_) = deserialized_response {
        let response_json = &BadRequestResponse {
            message: "There was an error uploading the campaign to ipfs".to_string(),
        };
        return Ok(warp::reply::with_status(
            json(response_json),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let deserialized_response = deserialized_response.unwrap();

    let response_json = &PublishSuccessResponse {
        status: "Campaign successfully uploaded to IPFS".to_string(),
        cid: deserialized_response.ipfs_hash,
    };
    return Ok(warp::reply::with_status(
        json(response_json),
        warp::http::StatusCode::OK,
    ));
}

pub fn build_route(
    db: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "publish" / String)
        .and(warp::post())
        .and(with_db(db))
        .and_then(publish_campaign_handler)
}
