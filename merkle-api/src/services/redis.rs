use dotenvy::dotenv;
use reqwest;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct RedisResponse {
    pub result: String,
}


pub async fn get_from_redis<Y: DeserializeOwned>(key: &str) -> Result<Option<Y>, reqwest::Error> {
    dotenv().ok();
    let redis_no_data_response = "{\"result\":null}";
    let redis_api_url = std::env::var("REDIS_API_URL").expect("REDIS_API_URL must be set");
    let redis_api_token = std::env::var("REDIS_API_TOKEN").expect("REDIS_API_TOKEN must be set");
    let url = format!("{}/get/{}", redis_api_url, key);
    let bearer_token = format!("Bearer {}", redis_api_token);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", bearer_token)
        .send()
        .await?;
    let text_response = response.text().await?;
    if text_response == redis_no_data_response {
        return Ok(None);
    }
    let redis_response: RedisResponse = serde_json::from_str(&text_response).unwrap();
    let data: Y = serde_json::from_str(&redis_response.result).unwrap();
    Ok(Some(data))
}

pub async fn set_in_redis<T: Serialize>(key: &str, data: &T) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let redis_api_url = std::env::var("REDIS_API_URL").expect("REDIS_API_URL must be set");
    let redis_api_token = std::env::var("REDIS_API_TOKEN").expect("REDIS_API_TOKEN must be set");
    let url = format!("{}/set/{}", redis_api_url, key);
    let bearer_token = format!("Bearer {}", redis_api_token);
    let serialized_data = json!(&data);

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Authorization", bearer_token)
        .json(&serialized_data)
        .send()
        .await?;
    let text_response = response.text().await?;
    Ok(text_response)
}
