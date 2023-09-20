use crate::{
    entities::{campaign, recipient},
    response::{
        BadRequestResponse, GenericResponse, UploadSuccessResponse, ValidationErrorResponse,
    },
    utils::{CsvData, CsvRecord},
    FormData, StreamExt, TryStreamExt, WebResult,
};
use bytes::BufMut;
use chrono::Utc;
use csv::ReaderBuilder;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, QueryFilter, QuerySelect, Set};
use sea_orm::{DbConn, EntityTrait};
use std::str;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{reply::json, Reply};

#[derive(Debug)]
struct CustomError(String);
impl warp::reject::Reject for CustomError {}

pub async fn health_checker_handler() -> WebResult<impl Reply> {
    const MESSAGE: &str = "Server up and running";

    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };
    Ok(json(response_json))
}

pub async fn upload_handler(form: FormData, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
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

                    let now = Utc::now();
                    let id = Uuid::new_v4();

                    let campaign = campaign::ActiveModel {
                        created_at: Set(now.timestamp().to_string()),
                        gid: Set(id.to_string()),
                        ..Default::default()
                    };

                    let campaign_model = campaign.insert(&db_conn).await;
                    match campaign_model {
                        Ok(campaign) => {
                            let recipient_inputs =
                                parsed_csv.records.iter().map(|rec| recipient::ActiveModel {
                                    address: Set(rec.address.clone()),
                                    amount: Set(rec.amount),
                                    campaign_id: Set(campaign.id),
                                    ..Default::default()
                                });
                            let recipients_model = recipient::Entity::insert_many(recipient_inputs)
                                .exec(&db_conn)
                                .await;

                            match recipients_model {
                                Ok(_) => {
                                    let recipients = recipient::Entity::find()
                                        .filter(
                                            Condition::any()
                                                .add(recipient::Column::CampaignId.eq(campaign.id)),
                                        )
                                        .offset(0)
                                        .limit(50)
                                        .all(&db_conn)
                                        .await;

                                    match recipients {
                                        Ok(rec) => {
                                            let lines: Vec<CsvRecord> = rec.into_iter().map(|x| CsvRecord {
                                                address: x.address,
                                                amount: x.amount,
                                            }).collect();
                                            let response_json = &UploadSuccessResponse {
                                                status: "Successfully uploaded file".to_string(),
                                                lines,
                                                page_number: 0,
                                                page_size: 50,
                                            };
                                            return Ok(warp::reply::with_status(
                                                json(response_json),
                                                warp::http::StatusCode::OK,
                                            ));
                                        }
                                        Err(_e) => {
                                            let response_json = &BadRequestResponse {
                                                message:
                                                    "There was a problem while creating a new campaign"
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
                                        message:
                                            "There was a problem while creating a new campaign"
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
