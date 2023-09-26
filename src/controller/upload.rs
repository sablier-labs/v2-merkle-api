use crate::{
    data_objects::dto::{CampaignDto, RecipientDto, RecipientPageDto},
    data_objects::response::{BadRequestResponse, UploadSuccessResponse, ValidationErrorResponse},
    repository,
    database::management::with_db,
    utils::csv::CsvData,
    FormData, StreamExt, TryStreamExt, WebResult,
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

async fn process_part(part: warp::multipart::Part) -> Result<CsvData, warp::Rejection> {
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
    let parsed_data =
        CsvData::build(rdr).map_err(|e| warp::reject::custom(CustomError(e.to_string())))?;

    Ok(parsed_data)
}

async fn upload_handler(form: FormData, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();
    let mut form = form;
    while let Some(Ok(part)) = form.next().await {
        let name = part.name();

        if name == "file.csv" {
            let parsed_csv = process_part(part).await;

            match parsed_csv {
                Ok(parsed_csv) => {
                    if parsed_csv.validation_errors.len() > 0 {
                        let response_json = &ValidationErrorResponse {
                            status: "Invalid csv file.".to_string(),
                            errors: parsed_csv.validation_errors,
                        };
                        return Ok(warp::reply::with_status(
                            json(response_json),
                            warp::http::StatusCode::BAD_REQUEST,
                        ));
                    }

                    let campaign_result = repository::campaign::create_campaign(parsed_csv.records, &db_conn).await;
                    match campaign_result {
                        Ok(campaign) => {
                            let recipient_result =
                                repository::recipient::get_recipients_by_campaign_id(campaign.id, 1, 50, &db_conn).await;
                            match recipient_result {
                                Ok(recipient) => {
                                    let response_json = &UploadSuccessResponse {
                                        status: "Upload successful".to_string(),
                                        campaign: CampaignDto {
                                            created_at: campaign.created_at,
                                            guid: campaign.guid,
                                            total_amount: campaign.total_amount,
                                            number_of_recipients: campaign.number_of_recipients,
                                        },
                                        page: RecipientPageDto {
                                            page_number: 1,
                                            page_size: 50,
                                            recipients: recipient
                                                .into_iter()
                                                .map(|x| RecipientDto {
                                                    address: x.address,
                                                    amount: x.amount,
                                                })
                                                .collect(),
                                        },
                                    };
                                    return Ok(warp::reply::with_status(
                                        json(response_json),
                                        warp::http::StatusCode::OK,
                                    ));
                                }
                                Err(_) => {
                                    let response_json = &BadRequestResponse {
                                        message:
                                            "There was a problem while fetching the recipients"
                                                .to_string(),
                                    };
                                    return Ok(warp::reply::with_status(
                                        json(response_json),
                                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                    ));
                                }
                            }
                        }
                        Err(_) => {
                            let response_json = &BadRequestResponse {
                                message: "There was a problem while creating a new campaign"
                                    .to_string(),
                            };
                            return Ok(warp::reply::with_status(
                                json(response_json),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            ));
                        }
                    }
                }
                Err(_e) => {
                    let response_json = &BadRequestResponse {
                        message: "There was a problem in csv file parsing process".to_string(),
                    };
                    return Ok(warp::reply::with_status(
                        json(response_json),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            }
        }
    }

    let response_json = &BadRequestResponse {
        message: "The request form data did not contain file.csv".to_string(),
    };
    return Ok(warp::reply::with_status(
        json(response_json),
        warp::http::StatusCode::BAD_REQUEST,
    ));
}

pub fn build_route(
    db: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000))
        .and(with_db(db))
        .and_then(upload_handler)
}
