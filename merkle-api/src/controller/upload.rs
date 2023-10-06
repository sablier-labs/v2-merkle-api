use crate::{
    csv_campaign_parser::CampaignCsvParsed,
    data_objects::dto::{RecipientDto, RecipientPageDto},
    data_objects::{
        dto::PersistentCampaignDto,
        response::{self, BadRequestResponse, UploadSuccessResponse, ValidationErrorResponse},
    },
    services::ipfs::{try_deserialize_pinata_response, upload_to_ipfs},
    FormData, StreamExt, TryStreamExt, WebResult,
};
use bytes::BufMut;
use csv::ReaderBuilder;
use merkle_tree_rs::standard::StandardMerkleTree;

use std::str;
use warp::{reply::json, Filter, Reply};

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

async fn upload_handler(decimals: usize, form: FormData) -> WebResult<impl Reply> {
    let mut form = form;
    while let Some(Ok(part)) = form.next().await {
        let name = part.name();

        if name == "file.csv" {
            let parsed_csv = process_part(part, decimals).await;
            if let Err(_) = parsed_csv {
                let response_json = &BadRequestResponse {
                    message: "There was a problem in csv file parsing process".to_string(),
                };

                return Ok(response::internal_server_error(json(response_json)));
            }

            let parsed_csv = parsed_csv.unwrap();
            if parsed_csv.validation_errors.len() > 0 {
                let response_json = &ValidationErrorResponse {
                    status: "Invalid csv file.".to_string(),
                    errors: parsed_csv.validation_errors,
                };
                return Ok(response::bad_request(json(response_json)));
            }

            let ipfs_response = upload_to_ipfs(PersistentCampaignDto {
                total_amount: parsed_csv.total_amount.to_string(),
                number_of_recipients: parsed_csv.number_of_recipients,
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
                let response_json = &BadRequestResponse {
                    message: "There was an error uploading the campaign to ipfs".to_string(),
                };
                return Ok(response::internal_server_error(json(response_json)));
            }

            let ipfs_response = ipfs_response.unwrap();
            let deserialized_response = try_deserialize_pinata_response(&ipfs_response);

            if let Err(_) = deserialized_response {
                let response_json = &BadRequestResponse {
                    message: "There was an error uploading the campaign to ipfs".to_string(),
                };
                return Ok(response::internal_server_error(json(response_json)));
            }

            let deserialized_response = deserialized_response.unwrap();

            let leaves = parsed_csv
                .records
                .iter()
                .enumerate()
                .map(|(i, r)| vec![i.to_string(), r.address.clone(), r.amount.to_string()])
                .collect();

            println!("Before tree: {:?}", std::time::SystemTime::now());

            let tree = StandardMerkleTree::of(
                leaves,
                &[
                    "uint".to_string(),
                    "address".to_string(),
                    "uint256".to_string(),
                ],
            );

            println!("After tree: {:?}", std::time::SystemTime::now());

            let response_json = &UploadSuccessResponse {
                status: "Upload successful".to_string(),
                total_amount: parsed_csv.total_amount,
                number_of_recipients: parsed_csv.number_of_recipients,
                root_hex: tree.root(),
                cid: deserialized_response.ipfs_hash,
                page: RecipientPageDto {
                    page_number: 1,
                    page_size: 50,
                    recipients: parsed_csv
                        .records
                        .into_iter()
                        .take(50)
                        .map(|x| RecipientDto {
                            address: x.address,
                            amount: x.amount.to_string(),
                        })
                        .collect(),
                },
            };

            println!("Before response: {:?}", std::time::SystemTime::now());
            return Ok(response::ok(json(response_json)));
        }
    }

    let response_json = &BadRequestResponse {
        message: "The request form data did not contain file.csv".to_string(),
    };
    return Ok(response::bad_request(json(response_json)));
}

type DecimalParam = usize;

pub fn build_route(
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "upload" / DecimalParam)
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000))
        .and_then(upload_handler)
}
