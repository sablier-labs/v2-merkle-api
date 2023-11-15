use crate::utils::csv_validator::ValidationError;
use serde::Serialize;
use serde_json::Value as Json;
use vercel_runtime as Vercel;
use warp::reply::WithStatus;

#[derive(Serialize, Debug)]
pub struct GeneralErrorResponse {
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
    pub root: String,
    pub total: String,
    pub recipients: String,
    pub cid: String,
}

#[derive(Serialize, Debug)]
pub struct EligibilityResponse {
    pub index: usize,
    pub proof: Vec<String>,
    pub address: String,
    pub amount: String,
}

#[derive(Serialize, Debug)]
pub struct ValidResponse {
    pub root: String,
    pub total: String,
    pub recipients: String,
    pub cid: String,
}

#[derive(Serialize, Debug)]
pub struct R {
    pub status: u16,
    pub message: Json,
}

pub fn bad_request(json_response: Json) -> R {
    R {
        status: warp::http::StatusCode::BAD_REQUEST.as_u16(),
        message: json_response,
    }
}

pub fn ok(json_response: Json) -> R {
    R {
        status: warp::http::StatusCode::OK.as_u16(),
        message: json_response,
    }
}

pub fn internal_server_error(json_response: Json) -> R {
    R {
        status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        message: json_response,
    }
}

pub fn to_warp(response: R) -> WithStatus<warp::reply::Json> {
    warp::reply::with_status(
        warp::reply::json(&response.message),
        warp::http::StatusCode::from_u16(response.status).unwrap(),
    )
}

pub fn to_vercel(response: R) -> Result<Vercel::Response<Vercel::Body>, Vercel::Error> {
    return Ok(Vercel::Response::builder()
        .status(response.status)
        .header("content-type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Methods",
            "GET, POST, PATCH, PUT, DELETE, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .body(response.message.to_string().into())?);
}
