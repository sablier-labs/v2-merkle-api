use dotenvy::dotenv;
use reqwest::{
    self,
    multipart::{Form, Part},
};

use serde_json::json;

use crate::data_objects::dto::PersistentCampaignDto;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::redis::{get_from_redis, set_in_redis};

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
    let part = Part::bytes(bytes)
        .file_name("data.json") // Adjust the filename if needed
        .mime_str("application/json")?; // Set MIME type to application/json

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
    let redis_data: Option<T> = get_from_redis(cid).await?;
    if let None = redis_data {
        println!("no redis data");
        let ipfs_url = format!("https://aqua-allied-falcon-825.mypinata.cloud/ipfs/{}", cid);
        let response = reqwest::get(&ipfs_url).await?;
        let data: T = response.json().await?;
        let d = set_in_redis(cid, &data).await;
        if let Err(e) = d {
            println!("{:?}", e);
        } else {
            let d = d.unwrap();
            println!("{:?}", d);
        }
        return Ok(data);
    }

    println!("redis data");
    let redis_data = redis_data.unwrap();
    Ok(redis_data)
}
