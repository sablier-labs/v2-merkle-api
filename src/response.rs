use crate::{
    dto::{CampaignDto, RecipientPageDto},
    utils::ValidationError,
};
use serde::Serialize;

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