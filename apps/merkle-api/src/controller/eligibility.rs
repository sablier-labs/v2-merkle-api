use crate::{
    data_objects::{
        dto::PersistentCampaignDto,
        query_param::Eligibility,
        response::{self, EligibilityResponse, GeneralErrorResponse},
    },
    services::ipfs::download_from_ipfs,
    WebResult,
};
use merkle_tree_rs::standard::{LeafType, StandardMerkleTree, StandardMerkleTreeData};

use serde_json::json;
use std::collections::HashMap;
use std::str;
use url::Url;

use vercel_runtime as Vercel;
use warp::Filter;

pub async fn handler(eligibility: Eligibility) -> response::R {
    let ipfs_data = download_from_ipfs::<PersistentCampaignDto>(&eligibility.cid).await;
    if let Err(_) = ipfs_data {
        let response_json = json!(GeneralErrorResponse {
            message: String::from("There was a problem processing your request."),
        });

        return response::internal_server_error(response_json);
    }
    let ipfs_data = ipfs_data.unwrap();
    let recipient_index = ipfs_data
        .recipients
        .iter()
        .position(|r| r.address.to_lowercase() == eligibility.address.to_lowercase());

    if let None = recipient_index {
        let response_json = json!(GeneralErrorResponse {
            message: String::from("The provided address is not eligible for this campaign"),
        });

        return response::bad_request(response_json);
    }

    let recipient_index = recipient_index.unwrap();

    let tree_data: StandardMerkleTreeData = serde_json::from_str(&ipfs_data.merkle_tree).unwrap();

    let tree = StandardMerkleTree::load(tree_data);

    let proof = tree.get_proof(LeafType::Number(recipient_index));

    let response_json = json!(&EligibilityResponse {
        index: recipient_index,
        proof,
        address: ipfs_data.recipients[recipient_index].address.clone(),
        amount: ipfs_data.recipients[recipient_index].amount.clone(),
    });
    return response::ok(response_json);
}

pub async fn handler_to_warp(eligibility: Eligibility) -> WebResult<impl warp::Reply> {
    let result = handler(eligibility).await;
    return Ok(response::to_warp(result));
}

pub async fn handler_to_vercel(
    req: Vercel::Request,
) -> Result<Vercel::Response<Vercel::Body>, Vercel::Error> {
    let url = Url::parse(&req.uri().to_string()).unwrap();
    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();

    let fallback = String::from("");

    let params = Eligibility {
        address: query.get("address").unwrap_or_else(|| &fallback).clone(),
        cid: query.get("cid").unwrap_or_else(|| &fallback).clone(),
    };

    let result = handler(params).await;

    return Ok(Vercel::Response::builder()
        .status(Vercel::StatusCode::OK)
        .header("content-type", "application/json")
        .body(result.message.to_string().into())?);
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "eligibility")
        .and(warp::get())
        .and(warp::query::query::<Eligibility>())
        .and_then(handler_to_warp)
}
