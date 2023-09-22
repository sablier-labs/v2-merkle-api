use crate::{
    data_objects::dto::CampaignDto,
    data_objects::response::{BadRequestResponse, CampaignSuccessResponse},
    repository::get_campaign_by_gid,
    WebResult, services::db::with_db,
};

use sea_orm::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::json, Reply, Filter};

async fn get_campaign_handler(gid: String, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();

    let campaign = get_campaign_by_gid(gid, &db_conn).await;
    match campaign {
        Ok(campaign) => match campaign {
            Some(campaign) => {
                let response_json = &CampaignSuccessResponse {
                    status: "Request successful".to_string(),
                    campaign: CampaignDto {
                        created_at: campaign.created_at,
                        total_amount: campaign.total_amount,
                        number_of_recipients: campaign.number_of_recipients,
                        gid: campaign.gid,
                    },
                };
                return Ok(warp::reply::with_status(
                    json(response_json),
                    warp::http::StatusCode::OK,
                ));
            }
            None => {
                let response_json = &BadRequestResponse {
                    message: "There is no campaign match the provided gid.".to_string(),
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
    warp::path!("api" / "campaigns" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_campaign_handler)
}
