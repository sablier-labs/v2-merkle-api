use crate::{
    data_objects::dto::CampaignDto,
    data_objects::response::{self, BadRequestResponse, CampaignSuccessResponse},
    database::management::with_db,
    repository, WebResult,
};

use sea_orm::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::json, Filter, Reply};

async fn get_campaign_handler(guid: String, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();

    let campaign = repository::campaign::get_campaign_by_guid(guid, &db_conn).await;

    if let Err(_) = campaign {
        let response_json = BadRequestResponse {
            message: "There was a problem processing your request.".to_string(),
        };

        return Ok(response::internal_server_error(json(&response_json)));
    }

    let campaign = campaign.unwrap();

    if let None = campaign {
        let response_json: BadRequestResponse = BadRequestResponse {
            message: "There is no campaign match the provided guid.".to_string(),
        };

        return Ok(response::bad_request(json(&response_json)));
    }

    let campaign = campaign.unwrap();
    let response_json = CampaignSuccessResponse {
        status: "Request successful".to_string(),
        campaign: CampaignDto {
            created_at: campaign.created_at,
            total_amount: campaign.total_amount,
            number_of_recipients: campaign.number_of_recipients,
            guid: campaign.guid,
        },
    };
    return Ok(response::ok(json(&response_json)));
}

pub fn build_route(
    db: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "campaigns" / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_campaign_handler)
}
