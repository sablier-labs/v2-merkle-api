use crate::database;
use migration::DbErr;
use sea_orm::{ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter, QuerySelect};

pub async fn get_recipients_by_campaign_id(
    campaign_id: i32,
    page_number: u64,
    page_size: u64,
    db_conn: &DbConn,
) -> Result<Vec<database::entity::recipient::Model>, DbErr> {
    let offset = (page_number - 1) * page_size;
    let recipients = database::entity::recipient::Entity::find()
        .filter(
            Condition::any().add(database::entity::recipient::Column::CampaignId.eq(campaign_id)),
        )
        .offset(offset)
        .limit(page_size)
        .all(db_conn)
        .await?;
    Ok(recipients)
}

pub async fn get_recipients_by_campaign_guid(
    campaign_gid: String,
    page_number: u64,
    page_size: u64,
    db_conn: &DbConn,
) -> Result<Vec<database::entity::recipient::Model>, DbErr> {
    let campaign = database::entity::campaign::Entity::find()
        .filter(Condition::any().add(database::entity::campaign::Column::Guid.eq(campaign_gid)))
        .one(db_conn)
        .await?;

    match campaign {
        Some(campaign) => {
            let offset = (page_number - 1) * page_size;
            let recipients = database::entity::recipient::Entity::find()
                .filter(
                    Condition::any()
                        .add(database::entity::recipient::Column::CampaignId.eq(campaign.id)),
                )
                .offset(offset)
                .limit(page_size)
                .all(db_conn)
                .await?;
            Ok(recipients)
        }
        None => {
            let empty: Vec<database::entity::recipient::Model> = Vec::new();
            return Ok(empty);
        }
    }
}
