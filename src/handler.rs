use crate::{
    dto::{CampaignDto, RecipientDto, RecipientPageDto},
    ipfs::{try_deserialize_pinata_response, upload_to_ipfs},
    param::Pagination,
    repository::{
        create_campaign, get_campaign_by_gid, get_publish_information,
        get_recipients_by_campaign_gid, get_recipients_by_campaign_id,
    },
    response::{
        BadRequestResponse, CampaignSuccessResponse, GenericResponse, PublishSuccessResponse,
        RecipientsSuccessResponse, UploadSuccessResponse, ValidationErrorResponse,
    },
    utils::CsvData,
    FormData, StreamExt, TryStreamExt, WebResult,
};
use bytes::BufMut;
use csv::ReaderBuilder;

use sea_orm::DbConn;
use std::str;
use std::sync::Arc;
use tokio::sync::Mutex;
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

                    let campaign_result = create_campaign(parsed_csv.records, &db_conn).await;
                    match campaign_result {
                        Ok(campaign) => {
                            let recipient_result =
                                get_recipients_by_campaign_id(campaign.id, 1, 50, &db_conn).await;
                            match recipient_result {
                                Ok(recipient) => {
                                    let response_json = &UploadSuccessResponse {
                                        status: "Upload successful".to_string(),
                                        campaign: CampaignDto {
                                            created_at: campaign.created_at,
                                            gid: campaign.gid,
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

pub async fn get_recipients_handler(
    gid: String,
    pagination: Pagination,
    db: Arc<Mutex<DbConn>>,
) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();

    let recipients =
        get_recipients_by_campaign_gid(gid, pagination.page_number, pagination.page_size, &db_conn)
            .await;
    match recipients {
        Ok(recipients) => {
            let response_json = &RecipientsSuccessResponse {
                status: "Request successful".to_string(),
                page: RecipientPageDto {
                    page_number: pagination.page_number,
                    page_size: pagination.page_size,
                    recipients: recipients
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
                message: "There was a problem processing your request.".to_string(),
            };
            return Ok(warp::reply::with_status(
                json(response_json),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
}

pub async fn get_campaign_handler(gid: String, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();

    let campaign = get_campaign_by_gid(gid, &db_conn).await;
    match campaign {
        Ok(campaign) => match campaign {
            Some(campaign) => {
                let response_json = &CampaignSuccessResponse {
                    status: "Request successful".to_string(),
                    campaign: CampaignDto {
                        created_at: campaign.created_at,
                        total_amount: campaign.total_amount,
                        number_of_recipients: campaign.number_of_recipients,
                        gid: campaign.gid,
                    },
                };
                return Ok(warp::reply::with_status(
                    json(response_json),
                    warp::http::StatusCode::OK,
                ));
            }
            None => {
                let response_json = &BadRequestResponse {
                    message: "There is no campaign match the provided gid.".to_string(),
                };
                return Ok(warp::reply::with_status(
                    json(response_json),
                    warp::http::StatusCode::BAD_REQUEST,
                ));
            }
        },
        Err(_) => {
            let response_json = &BadRequestResponse {
                message: "There was a problem processing your request.".to_string(),
            };
            return Ok(warp::reply::with_status(
                json(response_json),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
}

pub async fn publish_campaign(gid: String, db: Arc<Mutex<DbConn>>) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();
    let campaign_info = get_publish_information(gid, &db_conn).await;
    match campaign_info {
        Ok(campaign_info) => match campaign_info {
            Some(campaign_info) => {
                let ipfs_response = upload_to_ipfs(campaign_info).await;
                match ipfs_response {
                    Ok(ipfs_response) => {
                        let deserialized_response = try_deserialize_pinata_response(&ipfs_response);
                        match deserialized_response {
                            Ok(deserialized_response) => {
                                let response_json = &PublishSuccessResponse {
                                    status: "Campaign successfully uploaded to IPFS".to_string(),
                                    cid: deserialized_response.ipfs_hash,
                                };
                                return Ok(warp::reply::with_status(
                                    json(response_json),
                                    warp::http::StatusCode::OK,
                                ));
                            }
                            Err(_) => {
                                let response_json = &BadRequestResponse {
                                    message: "There was an error uploading the campaign to ipfs"
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
                            message: "There was an error uploading the campaign to ipfs"
                                .to_string(),
                        };
                        return Ok(warp::reply::with_status(
                            json(response_json),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ));
                    }
                }
            }
            None => {
                let response_json = &BadRequestResponse {
                    message: "Could not find a campaign with the specified gid".to_string(),
                };
                return Ok(warp::reply::with_status(
                    json(response_json),
                    warp::http::StatusCode::BAD_REQUEST,
                ));
            }
        },
        Err(_) => {
            let response_json = &BadRequestResponse {
                message: "There was a problem processing your request.".to_string(),
            };
            return Ok(warp::reply::with_status(
                json(response_json),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
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
