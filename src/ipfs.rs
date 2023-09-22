use dotenvy::dotenv;
use reqwest;
use serde_json::json;

use crate::dto::PersistentCampaignDto;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PinataSuccess {
    #[serde(rename = "IpfsHash")]
    pub ipfs_hash: String,
    #[serde(rename = "PinSize")]
    pub pin_size: usize,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
}

pub fn try_deserialize_pinata_response(
    response_body: &str,
) -> Result<PinataSuccess, serde_json::Error> {
    let success = serde_json::from_str::<PinataSuccess>(response_body)?;
    return Ok(success);
}

pub async fn upload_to_ipfs(data: PersistentCampaignDto) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let pinata_api_key = std::env::var("PINATA_API_KEI").expect("PINATA_API_KEI must be set");
    let pinata_secret_api_key =
        std::env::var("PINATA_SECRET_API_KEY").expect("PINATA_SECRET_API_KEY must be set");

    let client = reqwest::Client::new();

    let api_endpoint = "https://api.pinata.cloud/pinning/pinJSONToIPFS";

    let serialized_data = json!(&data);

    let response = client
        .post(api_endpoint)
        .header("pinata_api_key", pinata_api_key)
        .header("pinata_secret_api_key", pinata_secret_api_key)
        .json(&serialized_data)
        .send()
        .await?;

    let text_response = response.text().await?;
    Ok(text_response)
}
