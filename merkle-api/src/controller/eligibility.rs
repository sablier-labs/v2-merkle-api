use crate::{
    csv_campaign_parser::CampaignCsvRecord,
    data_objects::{
        dto::PersistentCampaignDto,
        query_param::Eligibility,
        response::{BadRequestResponse, EligibilityResponse},
    },
    services::ipfs::download_from_ipfs,
    utils::merkle::{HashingAlgorithm, SerializedProof},
    WebResult,
};
use merkle_light::merkle::MerkleTree;
use warp::{reply::json, Filter, Reply};

async fn get_eligibility_handler(eligibility: Eligibility) -> WebResult<impl Reply> {
    let ipfs_data = download_from_ipfs::<PersistentCampaignDto>(&eligibility.cid).await;
    if let Err(_) = ipfs_data {
        let response_json = &BadRequestResponse {
            message: "There was a problem processing your request.".to_string(),
        };
        return Ok(warp::reply::with_status(
            json(response_json),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }
    let ipfs_data = ipfs_data.unwrap();
    let recipient_index = ipfs_data
        .recipients
        .iter()
        .position(|r| r.address == eligibility.address);

    if let None = recipient_index {
        let response_json = &BadRequestResponse {
            message: "The provided address is not eligible for this campaign".to_string(),
        };
        return Ok(warp::reply::with_status(
            json(response_json),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let recipient_index = recipient_index.unwrap();
    let bytes: Vec<[u8; 32]> = ipfs_data
        .recipients
        .iter()
        .map(|r| CampaignCsvRecord {
            address: r.address.clone(),
            amount: r.amount,
        })
        .map(|r| r.to_hashed_bytes())
        .collect();

    let tree: MerkleTree<[u8; 32], HashingAlgorithm> = MerkleTree::from_iter(bytes);
    let proof = tree.gen_proof(recipient_index);
    let serialized_proof = SerializedProof::from_proof(&proof);

    let response_json = &EligibilityResponse {
        index: recipient_index,
        proof: serialized_proof,
    };
    return Ok(warp::reply::with_status(
        json(response_json),
        warp::http::StatusCode::OK,
    ));
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "eligibility")
        .and(warp::get())
        .and(warp::query::query::<Eligibility>())
        .and_then(get_eligibility_handler)
}
