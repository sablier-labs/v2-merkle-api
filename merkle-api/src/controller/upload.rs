use crate::{
    csv_campaign_parser::CampaignCsvParsed,
    data_objects::dto::{CampaignDto, RecipientDto, RecipientPageDto},
    data_objects::response::{BadRequestResponse, UploadSuccessResponse, ValidationErrorResponse, self},
    database::management::with_db,
    repository, FormData, StreamExt, TryStreamExt, WebResult,
};
use bytes::BufMut;
use csv::ReaderBuilder;

use sea_orm::DbConn;
use std::str;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::json, Filter, Reply};

#[derive(Debug)]
struct CustomError(String);
impl warp::reject::Reject for CustomError {}

async fn process_part(part: warp::multipart::Part, decimals: u32) -> Result<CampaignCsvParsed, warp::Rejection> {
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

async fn upload_handler(decimals: u32, form: FormData, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();
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
            let campaign_result = repository::campaign::create_campaign(
                parsed_csv.records,
                parsed_csv.total_amount,
                parsed_csv.number_of_recipients,
                &db_conn,
            )
            .await;

            if let Err(err) = campaign_result {
                println!("{}",err);
                let response_json = &BadRequestResponse {
                    message: "There was a problem while creating a new campaign".to_string(),
                };
                return Ok(response::internal_server_error(json(response_json)));
            }

            let campaign_result = campaign_result.unwrap();
            let recipient_result = repository::recipient::get_recipients_by_campaign_id(
                campaign_result.id,
                1,
                50,
                &db_conn,
            )
            .await;

            if let Err(_) = recipient_result {
                let response_json = &BadRequestResponse {
                    message: "There was a problem while fetching the recipients".to_string(),
                };
                return Ok(response::internal_server_error(json(response_json)));
            }

            let recipient_result = recipient_result.unwrap();
            let response_json = &UploadSuccessResponse {
                status: "Upload successful".to_string(),
                campaign: CampaignDto {
                    created_at: campaign_result.created_at,
                    guid: campaign_result.guid,
                    total_amount: campaign_result.total_amount,
                    number_of_recipients: campaign_result.number_of_recipients,
                },
                page: RecipientPageDto {
                    page_number: 1,
                    page_size: 50,
                    recipients: recipient_result
                        .into_iter()
                        .map(|x| RecipientDto {
                            address: x.address,
                            amount: x.amount,
                        })
                        .collect(),
                },
            };
            return Ok(response::ok(json(response_json)));
        }
    }

    let response_json = &BadRequestResponse {
        message: "The request form data did not contain file.csv".to_string(),
    };
    return Ok(response::bad_request(json(response_json)));
}

pub fn build_route(
    db: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "upload" / u32)
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000))
        .and(with_db(db))
        .and_then(upload_handler)
}
