use crate::{
    csv_campaign_parser::CampaignCsvParsed,
    data_objects::dto::RecipientDto,
    data_objects::{
        dto::PersistentCampaignDto,
        query_param::Create,
        response::{self, GeneralErrorResponse, UploadSuccessResponse, ValidationErrorResponse},
    },
    services::ipfs::{try_deserialize_pinata_response, upload_to_ipfs},
    FormData, StreamExt, TryStreamExt, WebResult,
};

use csv::ReaderBuilder;
use merkle_tree_rs::standard::StandardMerkleTree;
use std::{collections::HashMap, io::Read, str};
use url::Url;

use serde_json::json;
use vercel_runtime as Vercel;
use warp::{Buf, Filter};

#[derive(Debug)]
struct CustomError(String);
impl warp::reject::Reject for CustomError {}

async fn handler(params: Create, buffer: &[u8]) -> response::R {
    let rdr = ReaderBuilder::new().from_reader(buffer);
    let parsed_csv = CampaignCsvParsed::build(rdr, params.decimals);

    if let Err(error) = parsed_csv {
        let response_json = json!(GeneralErrorResponse {
            message: format!(
                "There was a problem in csv file parsing process: {}",
                error.to_string()
            ),
        });

        return response::internal_server_error(response_json);
    }

    let parsed_csv = parsed_csv.unwrap();
    if parsed_csv.validation_errors.len() > 0 {
        let response_json = json!(ValidationErrorResponse {
            status: String::from("Invalid csv file."),
            errors: parsed_csv.validation_errors,
        });

        return response::bad_request(response_json);
    }

    let leaves = parsed_csv
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| vec![i.to_string(), r.address.clone(), r.amount.to_string()])
        .collect();

    let tree = StandardMerkleTree::of(
        leaves,
        &[
            "uint".to_string(),
            "address".to_string(),
            "uint256".to_string(),
        ],
    );

    let tree_json = serde_json::to_string(&tree.dump()).unwrap();

    let ipfs_response = upload_to_ipfs(PersistentCampaignDto {
        total_amount: parsed_csv.total_amount.to_string(),
        number_of_recipients: parsed_csv.number_of_recipients,
        merkle_tree: tree_json,
        recipients: parsed_csv
            .records
            .iter()
            .map(|x| RecipientDto {
                address: x.address.clone(),
                amount: x.amount.to_string(),
            })
            .collect(),
    })
    .await;
    if let Err(_) = ipfs_response {
        let response_json = json!(GeneralErrorResponse {
            message: String::from("There was an error uploading the campaign to ipfs"),
        });

        return response::internal_server_error(response_json);
    }

    let ipfs_response = ipfs_response.unwrap();
    let deserialized_response = try_deserialize_pinata_response(&ipfs_response);

    if let Err(_) = deserialized_response {
        let response_json = json!(GeneralErrorResponse {
            message: String::from("There was an error uploading the campaign to ipfs"),
        });

        return response::internal_server_error(response_json);
    }

    let deserialized_response = deserialized_response.unwrap();

    let response_json = json!(UploadSuccessResponse {
        status: "Upload successful".to_string(),
        total: parsed_csv.total_amount.to_string(),
        recipients: parsed_csv.number_of_recipients.to_string(),
        root: tree.root(),
        cid: deserialized_response.ipfs_hash,
    });

    return response::ok(response_json);
}

pub async fn handler_to_warp(params: Create, form: FormData) -> WebResult<impl warp::Reply> {
    let mut form = form;
    while let Some(Ok(part)) = form.next().await {
        let name = part.name();

        if name == "data" {
            let mut stream = part.stream();
            let mut buffer = Vec::new();

            while let Ok(Some(chunk)) = stream.try_next().await {
                chunk.reader().read_to_end(&mut buffer).unwrap();
            }

            let result = handler(params, &buffer).await;
            return Ok(response::to_warp(result));
        }
    }

    let response_json = json!(GeneralErrorResponse {
        message: "The request form data did not contain recipients csv file".to_string(),
    });
    return Ok(response::to_warp(response::bad_request(response_json)));
}

pub async fn handler_to_vercel(
    req: Vercel::Request,
) -> Result<Vercel::Response<Vercel::Body>, Vercel::Error> {
    // ------------------------------------------------------------
    // Extract query parameters from the URL: decimals
    // ------------------------------------------------------------

    let url = Url::parse(&req.uri().to_string()).unwrap();
    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();
    let decimals = query.get("decimals");

    if let None = decimals {
        let response_json = json!(GeneralErrorResponse {
            message: String::from("The provided address is not eligible for this campaign"),
        });

        return response::to_vercel(response::ok(response_json));
    }

    // ------------------------------------------------------------
    // Extract form data from the body: file
    // ------------------------------------------------------------

    let boundary = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("multipart/form-data; boundary="));

    if let None = boundary {
        let response_json = json!(GeneralErrorResponse {
            message: String::from("Invalid content type header"),
        });

        return response::to_vercel(response::ok(response_json));
    }

    let boundary = boundary.unwrap();
    let body = req.body().to_vec();

    let mut data = multipart::server::Multipart::with_body(body.as_slice(), boundary);
    let file = data.read_entry();
    if let Err(error) = file {
        let response_json = json!(GeneralErrorResponse {
            message: String::from(error.to_string()),
        });

        return response::to_vercel(response::ok(response_json));
    }

    let file = file.unwrap();

    if let None = file {
        let response_json = json!(GeneralErrorResponse {
            message: String::from("Invalid form data, missing file"),
        });

        return response::to_vercel(response::ok(response_json));
    }

    let mut file = file.unwrap();
    let mut buffer: Vec<u8> = vec![];

    if let Err(error) = file.data.read_to_end(&mut buffer) {
        let response_json = json!(GeneralErrorResponse {
            message: format!("Could not read body data {}", error.to_string()),
        });

        return response::to_vercel(response::ok(response_json));
    }

    // ------------------------------------------------------------
    // Format arguments for the generic handler
    // ------------------------------------------------------------

    let decimals: u16 = decimals.unwrap().parse().unwrap_or_default();
    let create = Create {
        decimals: decimals.into(),
    };

    let result = handler(create, &buffer).await;
    return Ok(Vercel::Response::builder()
        .status(Vercel::StatusCode::OK)
        .header("content-type", "application/json")
        .body(result.message.to_string().into())?);
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "create")
        .and(warp::post())
        .and(warp::query::query::<Create>())
        .and(warp::multipart::form().max_length(100_000_000))
        .and_then(handler_to_warp)
}
