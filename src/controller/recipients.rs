use crate::{
    data_objects::dto::{RecipientDto, RecipientPageDto},
    data_objects::query_param::Pagination,
    data_objects::response::{BadRequestResponse, RecipientsSuccessResponse},
    repository,
    services::db::with_db,
    WebResult,
};

use sea_orm::DbConn;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{reply::json, Filter, Reply};

async fn get_recipients_handler(
    gid: String,
    pagination: Pagination,
    db: Arc<Mutex<DbConn>>,
) -> WebResult<impl Reply> {
    let db = db.lock().await;
    let db_conn = db.clone();

    let recipients =
        repository::recipient::get_recipients_by_campaign_guid(gid, pagination.page_number, pagination.page_size, &db_conn)
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

pub fn build_route(
    db: Arc<Mutex<DbConn>>,
) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "entries" / String)
        .and(warp::get())
        .and(warp::query::query::<Pagination>())
        .and(with_db(db))
        .and_then(get_recipients_handler)
}
