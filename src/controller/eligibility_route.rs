use crate::{
    data_objects::{
        dto::PersistentCampaignDto,
        response::{BadRequestResponse, EligibilityResponse},
    },
    services::ipfs::download_from_ipfs,
    WebResult,
};
use warp::{reply::json, Filter, Reply};

async fn get_eligibility_handler(cid: String) -> WebResult<impl Reply> {
    let ipfs_data = download_from_ipfs::<PersistentCampaignDto>(&cid).await;
    match ipfs_data {
        Ok(ipfs_data) => {
            let response_json = &EligibilityResponse {
                total_amount: ipfs_data.total_amount,
                number_of_recipients: ipfs_data.number_of_recipients,
            };
            return Ok(warp::reply::with_status(
                json(response_json),
                warp::http::StatusCode::OK,
            ));
        }
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
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "eligibility" / String)
        .and(warp::get())
        .and_then(get_eligibility_handler)
}
