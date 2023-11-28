use dotenvy::dotenv;
use reqwest::{
    self,
    multipart::{Form, Part},
};

use serde_json::json;

use crate::data_objects::dto::PersistentCampaignDto;
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Deserialize, Debug)]
pub struct PinataSuccess {
    #[serde(rename = "IpfsHash")]
    pub ipfs_hash: String,
    #[serde(rename = "PinSize")]
    pub pin_size: usize,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
}

pub fn try_deserialize_pinata_response(response_body: &str) -> Result<PinataSuccess, serde_json::Error> {
    let success = serde_json::from_str::<PinataSuccess>(response_body)?;
    Ok(success)
}

pub async fn upload_to_ipfs(data: PersistentCampaignDto) -> Result<String, reqwest::Error> {
    dotenv().ok();
    let pinata_api_key = std::env::var("PINATA_API_KEY").expect("PINATA_API_KEY must be set");
    let pinata_secret_api_key = std::env::var("PINATA_SECRET_API_KEY").expect("PINATA_SECRET_API_KEY must be set");
    let pinata_api_server = std::env::var("PINATA_API_SERVER").expect("PINATA_API_SERVER must be set");

    let client = reqwest::Client::new();

    let api_endpoint = format!("{}/pinning/pinFileToIPFS", pinata_api_server);

    let serialized_data = json!(&data);
    let bytes = serde_json::to_vec(&serialized_data).unwrap();
    let part = Part::bytes(bytes).file_name("data.json").mime_str("application/json")?;

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

pub async fn download_from_ipfs<T: DeserializeOwned>(cid: &str) -> Result<T, reqwest::Error> {
    dotenv().ok();
    let ipfs_gateway = std::env::var("IPFS_GATEWAY").expect("IPFS_GATEWAY must be set");
    let pinata_access_token = std::env::var("PINATA_ACCESS_TOKEN").expect("PINATA_ACCESS_TOKEN must be set");
    let ipfs_url = format!("{}{}?pinataGatewayToken={}", ipfs_gateway, cid, pinata_access_token);
    let response = reqwest::get(&ipfs_url).await?;
    let data: T = response.json().await?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_deserialize_pinata_response_success() {
        let result: Result<PinataSuccess, serde_json::Error> = try_deserialize_pinata_response(
            r#"{"IpfsHash": "test_hash", "PinSize": 123, "Timestamp": "2023-04-05T00:00:00Z"}"#,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn try_deserialize_pinata_response_fail() {
        let result: Result<PinataSuccess, serde_json::Error> = try_deserialize_pinata_response("Error message");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upload_to_ipfs_ok() {
        let mut server = mockito::Server::new();

        let host = server.host_with_port();
        let parts: Vec<&str> = host.split(':').collect();
        let port = parts[1];
        let server_host = format!("http://localhost:{}", port);

        // Set environment variables
        std::env::set_var("PINATA_API_KEY", "mock_pinata_api_key");
        std::env::set_var("PINATA_SECRET_API_KEY", "mock_pinata_secret_key");
        std::env::set_var("PINATA_API_SERVER", server_host);

        // Set up mock server
        let mock = server
            .mock("POST", "/pinning/pinFileToIPFS")
            .with_status(200)
            .with_body(r#"{"IpfsHash": "test_hash", "PinSize": 123, "Timestamp": "2021-01-01T00:00:00Z"}"#)
            .create();

        // Call the function with a test data object
        let data = PersistentCampaignDto {
            total_amount: "128".to_string(),
            number_of_recipients: 4,
            root: "test_root".to_string(),
            merkle_tree: "test_merkle".to_string(),
            recipients: Vec::new(),
        };
        let result = upload_to_ipfs(data).await;

        assert!(result.is_ok());
        mock.assert();
    }

    #[tokio::test]
    async fn test_upload_to_ipfs_error() {
        let mut server = mockito::Server::new();

        let host = server.host_with_port();
        let parts: Vec<&str> = host.split(':').collect();
        let port = parts[1];
        let server_host = format!("http://localhost:{}", port);

        // Set environment variables
        std::env::set_var("PINATA_API_KEY", "mock_pinata_api_key");
        std::env::set_var("PINATA_SECRET_API_KEY", "mock_pinata_secret_key");
        std::env::set_var("PINATA_API_SERVER", server_host);

        // Set up mock server
        let mock = server
            .mock("POST", "/pinning/pinFileToIPFS")
            .with_status(500)
            .with_body(r#"{"code": "500", "message": "Internal server error"}"#)
            .create();

        // Call the function with a test data object
        let data = PersistentCampaignDto {
            total_amount: "128".to_string(),
            number_of_recipients: 4,
            root: "test_root".to_string(),
            merkle_tree: "test_merkle".to_string(),
            recipients: Vec::new(),
        };
        let result = upload_to_ipfs(data).await;

        let result = result.unwrap();
        let deserialized_response = try_deserialize_pinata_response(&result);
        assert!(deserialized_response.is_err());
        mock.assert();
    }

    #[tokio::test]
    async fn test_download_from_ipfs_success() {
        let mut server = mockito::Server::new_with_port(8000);

        let host = server.host_with_port();
        let parts: Vec<&str> = host.split(':').collect();
        let port = parts[1];
        let server_host = format!("http://localhost:{}/", port);

        // Set environment variables
        std::env::set_var("PINATA_ACCESS_TOKEN", "mock_pinata_access_token");
        std::env::set_var("IPFS_GATEWAY", server_host);

        // Set up mock server
        let mock = server
            .mock("GET", "/valid_cid?pinataGatewayToken=mock_pinata_access_token")
            .with_status(200)
            .with_body(r#"{"IpfsHash": "test_hash", "PinSize": 123, "Timestamp": "2021-01-01T00:00:00Z"}"#)
            .create();

        let result: Result<PinataSuccess, _> = download_from_ipfs("valid_cid").await;
        assert!(result.is_ok());
        mock.assert();
    }

    #[tokio::test]
    async fn test_download_from_ipfs_error() {
        let mut server = mockito::Server::new_with_port(8000);

        let host = server.host_with_port();
        let parts: Vec<&str> = host.split(':').collect();
        let port = parts[1];
        let server_host = format!("http://localhost:{}/", port);

        // Set environment variables
        std::env::set_var("PINATA_ACCESS_TOKEN", "mock_pinata_access_token");
        std::env::set_var("IPFS_GATEWAY", server_host);

        // Set up mock server
        let mock = server
            .mock("GET", "/valid_cid?pinataGatewayToken=mock_pinata_access_token")
            .with_status(500)
            .with_body(r#"{"code": "500", "message": "Internal server error"}"#)
            .create();

        let result: Result<PinataSuccess, _> = download_from_ipfs("valid_cid").await;
        assert!(result.is_err());
        mock.assert();
    }
}
