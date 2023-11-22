use crate::{
    data_objects::{
        dto::PersistentCampaignDto,
        query_param::Validity,
        response::{self, GeneralErrorResponse, ValidResponse},
    },
    services::ipfs::download_from_ipfs,
    WebResult,
};

use serde_json::json;
use std::collections::HashMap;
use std::str;
use url::Url;

use vercel_runtime as Vercel;
use warp::Filter;

pub async fn handler(validity: Validity) -> response::R {
    let ipfs_data = download_from_ipfs::<PersistentCampaignDto>(&validity.cid).await;
    if let Err(_) = ipfs_data {
        let response_json = json!(GeneralErrorResponse {
            message: format!("Bad CID or invalid file format provided."),
        });

        return response::internal_server_error(response_json);
    }
    let ipfs_data = ipfs_data.unwrap();

    let response_json = json!(&ValidResponse {
        root: ipfs_data.root,
        total: ipfs_data.total_amount,
        recipients: ipfs_data.number_of_recipients.to_string(),
        cid: validity.cid
    });
    return response::ok(response_json);
}

pub async fn handler_to_warp(validity: Validity) -> WebResult<impl warp::Reply> {
    let result = handler(validity).await;
    return Ok(response::to_warp(result));
}

pub async fn handler_to_vercel(
    req: Vercel::Request,
) -> Result<Vercel::Response<Vercel::Body>, Vercel::Error> {
    // ------------------------------------------------------------
    // Extract query parameters from the URL: address, cid
    // ------------------------------------------------------------

    let url = Url::parse(&req.uri().to_string()).unwrap();
    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();

    // ------------------------------------------------------------
    //Format arguments for the generic handler
    // ------------------------------------------------------------

    let fallback = String::from("");
    let params = Validity {
        cid: query.get("cid").unwrap_or_else(|| &fallback).clone(),
    };

    let result = handler(params).await;

    return response::to_vercel(result);
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "validity")
        .and(warp::get())
        .and(warp::query::query::<Validity>())
        .and_then(handler_to_warp)
}
