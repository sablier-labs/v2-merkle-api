use crate::{
    entities::{campaign, recipient},
    utils::CsvRecord,
};
use chrono::Utc;
use migration::DbErr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

pub async fn create_campaign(
    records: Vec<CsvRecord>,
    db_conn: &DbConn,
) -> Result<campaign::Model, DbErr> {
    let now = Utc::now();
    let id = Uuid::new_v4();

    let campaign = campaign::ActiveModel {
        created_at: Set(now.timestamp().to_string()),
        gid: Set(id.to_string()),
        ..Default::default()
    };
    let campaign_model = campaign.insert(db_conn).await?;

    let recipient_inputs = records.iter().map(|rec| recipient::ActiveModel {
        address: Set(rec.address.clone()),
        amount: Set(rec.amount),
        campaign_id: Set(campaign_model.id),
        ..Default::default()
    });

    let _recipients_model = recipient::Entity::insert_many(recipient_inputs)
        .exec(db_conn)
        .await?;
    Ok(campaign_model)
}

pub async fn get_recipients_by_campaign_id(
    campaign_id: i32,
    page_number: u64,
    page_size: u64,
    db_conn: &DbConn,
) -> Result<Vec<recipient::Model>, DbErr> {
    let offset = (page_number - 1) * page_size;
    let recipients = recipient::Entity::find()
        .filter(Condition::any().add(recipient::Column::CampaignId.eq(campaign_id)))
        .offset(offset)
        .limit(page_size)
        .all(db_conn)
        .await?;
    Ok(recipients)
}

pub async fn get_recipients_by_campaign_gid(
    campaign_gid: String,
    page_number: u64,
    page_size: u64,
    db_conn: &DbConn,
) -> Result<Vec<recipient::Model>, DbErr> {
    let campaign = campaign::Entity::find()
        .filter(Condition::any().add(campaign::Column::Gid.eq(campaign_gid)))
        .one(db_conn)
        .await?;

    match campaign {
        Some(campaign) => {
            let offset = (page_number - 1) * page_size;
            let recipients = recipient::Entity::find()
                .filter(Condition::any().add(recipient::Column::CampaignId.eq(campaign.id)))
                .offset(offset)
                .limit(page_size)
                .all(db_conn)
                .await?;
            Ok(recipients)
        }
        None => {
            let empty: Vec<recipient::Model> = Vec::new();
            return Ok(empty);
        }
    }
}
