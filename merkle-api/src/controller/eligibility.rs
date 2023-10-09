use crate::{
    data_objects::{
        dto::PersistentCampaignDto,
        query_param::Eligibility,
        response::{self, BadRequestResponse, EligibilityResponse},
    },
    services::ipfs::download_from_ipfs,
    WebResult,
};
use merkle_tree_rs::standard::{LeafType, StandardMerkleTree, StandardMerkleTreeData};
use warp::{reply::json, Filter, Reply};

async fn get_eligibility_handler(eligibility: Eligibility) -> WebResult<impl Reply> {
    let ipfs_data = download_from_ipfs::<PersistentCampaignDto>(&eligibility.cid).await;
    if let Err(_) = ipfs_data {
        let response_json = &BadRequestResponse {
            message: "There was a problem processing your request.".to_string(),
        };

        return Ok(response::internal_server_error(json(response_json)));
    }
    let ipfs_data = ipfs_data.unwrap();
    let recipient_index = ipfs_data
        .recipients
        .iter()
        .position(|r| r.address.to_lowercase() == eligibility.address.to_lowercase());

    if let None = recipient_index {
        let response_json = &BadRequestResponse {
            message: "The provided address is not eligible for this campaign".to_string(),
        };
        return Ok(response::bad_request(json(response_json)));
    }

    let recipient_index = recipient_index.unwrap();

    let tree_data: StandardMerkleTreeData = serde_json::from_str(&ipfs_data.merkle_tree).unwrap();

    let tree = StandardMerkleTree::load(tree_data);

    let proof = tree.get_proof(LeafType::Number(recipient_index));

    let response_json = &EligibilityResponse {
        index: recipient_index,
        proof,
        address: ipfs_data.recipients[recipient_index].address.clone(),
        amount: ipfs_data.recipients[recipient_index].amount.clone(),
    };
    return Ok(response::ok(json(response_json)));
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "eligibility")
        .and(warp::get())
        .and(warp::query::query::<Eligibility>())
        .and_then(get_eligibility_handler)
}
