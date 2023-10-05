use dotenvy::dotenv;
use reqwest::{
    self,
    multipart::{Form, Part},
};
use std::io::Cursor;

use serde_json::json;

use crate::data_objects::dto::PersistentCampaignDto;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use zstd::stream::{decode_all, encode_all};

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
    println!("response body: {}", response_body);
    let success = serde_json::from_str::<PinataSuccess>(response_body)?;
    return Ok(success);
}

pub async fn upload_to_ipfs(data: PersistentCampaignDto) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let pinata_api_key = std::env::var("PINATA_API_KEI").expect("PINATA_API_KEI must be set");
    let pinata_secret_api_key =
        std::env::var("PINATA_SECRET_API_KEY").expect("PINATA_SECRET_API_KEY must be set");

    let client = reqwest::Client::new();

    let api_endpoint = "https://api.pinata.cloud/pinning/pinFileToIPFS";

    let serialized_data = json!(&data);
    let bytes = serde_json::to_vec(&serialized_data).unwrap(); // Convert the JSON value to bytes
    let compressed_data = encode_all(Cursor::new(bytes), 0).expect("Error at compressing data");

    let part = Part::bytes(compressed_data)
        .file_name("data.json.zst")
        .mime_str("application/octet-stream")?;

    let form = Form::new().part("file", part);

    let response = client
        .post(api_endpoint)
        .header("pinata_api_key", pinata_api_key)
        .header("pinata_secret_api_key", pinata_secret_api_key)
        .multipart(form)
        .send()
        .await?;

    let text_response = response.text().await?;
    Ok(text_response)
}

pub async fn download_from_ipfs<T: DeserializeOwned + Serialize>(
    cid: &str,
) -> Result<T, reqwest::Error> {
    dotenv().ok();
    let ipfs_gateway = std::env::var("IPFS_GATEWAY").expect("IPFS_GATEWAY must be set");
    let ipfs_url = format!("{}{}", ipfs_gateway, cid);
    let response = reqwest::get(&ipfs_url).await?;
    let compressed_bytes = response.bytes().await?;
    let decompressed_data =
        decode_all(Cursor::new(compressed_bytes)).expect("Error at decompressing file");
    let data: T = serde_json::from_slice(&decompressed_data).expect("Error at deserialize");
    Ok(data)
}
