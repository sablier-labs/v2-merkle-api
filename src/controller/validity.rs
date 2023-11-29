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
use std::{collections::HashMap, str};
use url::Url;

use vercel_runtime as Vercel;
use warp::Filter;

pub async fn handler(validity: Validity) -> response::R {
    let ipfs_data = download_from_ipfs::<PersistentCampaignDto>(&validity.cid).await;
    if ipfs_data.is_err() {
        let response_json =
            json!(GeneralErrorResponse { message: "Bad CID or invalid file format provided.".to_string() });

        return response::internal_server_error(response_json);
    }
    let ipfs_data = ipfs_data.unwrap();

    let response_json = json!(&ValidResponse {
        root: ipfs_data.root,
        total: ipfs_data.total_amount,
        recipients: ipfs_data.number_of_recipients.to_string(),
        cid: validity.cid
    });
    response::ok(response_json)
}

pub async fn handler_to_warp(validity: Validity) -> WebResult<impl warp::Reply> {
    let result = handler(validity).await;
    Ok(response::to_warp(result))
}

pub async fn handler_to_vercel(req: Vercel::Request) -> Result<Vercel::Response<Vercel::Body>, Vercel::Error> {
    // ------------------------------------------------------------
    // Extract query parameters from the URL: address, cid
    // ------------------------------------------------------------

    let url = Url::parse(&req.uri().to_string()).unwrap();
    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();

    // ------------------------------------------------------------
    //Format arguments for the generic handler
    // ------------------------------------------------------------

    let fallback = String::from("");
    let params = Validity { cid: query.get("cid").unwrap_or(&fallback).clone() };

    let result = handler(params).await;

    response::to_vercel(result)
}

pub fn build_route() -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "validity").and(warp::get()).and(warp::query::query::<Validity>()).and_then(handler_to_warp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handler_success_response() {
        let mut server = mockito::Server::new_with_port(8000);

        let host = server.host_with_port();
        let parts: Vec<&str> = host.split(':').collect();
        let port = parts[1];
        let server_host = format!("http://localhost:{}/", port);

        std::env::set_var("PINATA_ACCESS_TOKEN", "mock_pinata_access_token");
        std::env::set_var("IPFS_GATEWAY", server_host);

        let mock = server
            .mock("GET", "/valid_cid?pinataGatewayToken=mock_pinata_access_token")
            .with_status(200)
            .with_body(r#"{"root": "root", "total_amount": "123", "number_of_recipients": 3, "merkle_tree":"asd", "recipients": []}"#)
            .create();

        let validity = Validity { cid: "valid_cid".to_string() };
        let response = handler(validity).await;
        assert_eq!(response.status, warp::http::StatusCode::OK.as_u16());
        mock.assert();
    }

    #[tokio::test]
    async fn handler_error_response() {
        let mut server = mockito::Server::new_with_port(8000);

        let host = server.host_with_port();
        let parts: Vec<&str> = host.split(':').collect();
        let port = parts[1];
        let server_host = format!("http://localhost:{}/", port);

        std::env::set_var("PINATA_ACCESS_TOKEN", "mock_pinata_access_token");
        std::env::set_var("IPFS_GATEWAY", server_host);

        let mock = server
            .mock("GET", "/invalid_cid?pinataGatewayToken=mock_pinata_access_token")
            .with_status(500)
            .with_body(r#"{"message": "Bad request"}"#)
            .create();

        let validity = Validity { cid: "invalid_cid".to_string() };
        let response = handler(validity).await;
        assert_eq!(response.status, warp::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        mock.assert();
    }
}
