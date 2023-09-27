use crate::{
    csv_campaign_parser::ValidationError,
    data_objects::dto::{CampaignDto, RecipientPageDto},
    utils::merkle::SerializedProof,
};
use serde::Serialize;
use warp::reply::{Json, WithStatus};

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct BadRequestResponse {
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct ValidationErrorResponse {
    pub status: String,
    pub errors: Vec<ValidationError>,
}

#[derive(Serialize, Debug)]
pub struct UploadSuccessResponse {
    pub status: String,
    pub campaign: CampaignDto,
    pub page: RecipientPageDto,
}

#[derive(Serialize, Debug)]
pub struct RecipientsSuccessResponse {
    pub status: String,
    pub page: RecipientPageDto,
}

#[derive(Serialize, Debug)]
pub struct CampaignSuccessResponse {
    pub status: String,
    pub campaign: CampaignDto,
}

#[derive(Serialize, Debug)]
pub struct PublishSuccessResponse {
    pub status: String,
    pub cid: String,
}

#[derive(Serialize, Debug)]
pub struct EligibilityResponse {
    pub index: usize,
    pub proof: SerializedProof,
}

pub fn bad_request(json_response: Json) -> WithStatus<warp::reply::Json> {
    warp::reply::with_status(json_response, warp::http::StatusCode::BAD_REQUEST)
}

pub fn ok(json_response: Json) -> WithStatus<warp::reply::Json> {
    warp::reply::with_status(json_response, warp::http::StatusCode::OK)
}

pub fn internal_server_error(json_response: Json) -> WithStatus<warp::reply::Json> {
    warp::reply::with_status(json_response, warp::http::StatusCode::INTERNAL_SERVER_ERROR)
}
