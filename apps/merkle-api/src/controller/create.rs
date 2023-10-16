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
use bytes::BufMut;
use csv::ReaderBuilder;
use merkle_tree_rs::standard::StandardMerkleTree;

use std::str;

use serde_json::json;
use vercel_runtime as Vercel;
use warp::Filter;

#[derive(Debug)]
struct CustomError(String);
impl warp::reject::Reject for CustomError {}

async fn process_part(
    part: warp::multipart::Part,
    decimals: usize,
) -> Result<CampaignCsvParsed, warp::Rejection> {
    let value = part.stream();
    let data: Vec<u8> = value
        .try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        })
        .await
        .map_err(|e| warp::reject::custom(CustomError(e.to_string())))?;

    // Convert the bytes to a &str
    let s = str::from_utf8(&data).map_err(|e| warp::reject::custom(CustomError(e.to_string())))?;

    // Create a CSV reader
    let rdr = ReaderBuilder::new().from_reader(s.as_bytes());
    let parsed_data = CampaignCsvParsed::build(rdr, decimals)
        .map_err(|e| warp::reject::custom(CustomError(e.to_string())))?;

    Ok(parsed_data)
}

async fn handler(params: Create, form: FormData) -> response::R {
    let mut form = form;
    while let Some(Ok(part)) = form.next().await {
        let name = part.name();

        if name == "data" {
            let parsed_csv = process_part(part, params.decimals).await;
            if let Err(_) = parsed_csv {
                let response_json = json!(GeneralErrorResponse {
                    message: String::from("There was a problem in csv file parsing process"),
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
    }

    let response_json = json!(GeneralErrorResponse {
        message: "The request form data did not contain recipients csv file".to_string(),
    });
    return response::bad_request(response_json);
}

pub async fn handler_to_warp(params: Create, form: FormData) -> WebResult<impl warp::Reply> {
    let result = handler(params, form).await;
    return Ok(response::to_warp(result));
}

pub async fn handler_to_vercel(
    _req: Vercel::Request,
) -> Result<Vercel::Response<Vercel::Body>, Vercel::Error> {
    let result = json!({
        "status": "to do".to_string(),
        "message":"TO DO: convert feature to also work with vercel".to_string(),
    });

    let prepared = response::ok(result);

    return Ok(Vercel::Response::builder()
        .status(Vercel::StatusCode::OK)
        .header("content-type", "application/json")
        .body(prepared.message.to_string().into())?);
}

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "create")
        .and(warp::post())
        .and(warp::query::query::<Create>())
        .and(warp::multipart::form().max_length(100_000_000))
        .and_then(handler_to_warp)
}
