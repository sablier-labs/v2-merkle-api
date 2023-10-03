use crate::{
    csv_campaign_parser::CampaignCsvRecord,
    data_objects::dto::{PersistentCampaignDto, RecipientDto},
    database,
};
use chrono::Utc;
use migration::DbErr;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn create_campaign(
    records: Vec<CampaignCsvRecord>,
    total_amount: u128,
    number_of_recipients: i32,
    db_conn: &DbConn,
) -> Result<database::entity::campaign::Model, DbErr> {
    let now = Utc::now();
    let id = Uuid::new_v4();

    let campaign = database::entity::campaign::ActiveModel {
        created_at: Set(now.timestamp()),
        guid: Set(id.to_string()),
        total_amount: Set(total_amount.to_string()),
        number_of_recipients: Set(number_of_recipients),
        ..Default::default()
    };
    let campaign_model = campaign.insert(db_conn).await?;

    for chunk in records.chunks(100) {
        let recipient_inputs =
            chunk
                .into_iter()
                .map(|rec| database::entity::recipient::ActiveModel {
                    address: Set(rec.address.clone()),
                    amount: Set(rec.amount.to_string()),
                    campaign_id: Set(campaign_model.id),
                    ..Default::default()
                });

        let _recipients_model = database::entity::recipient::Entity::insert_many(recipient_inputs)
            .exec(db_conn)
            .await?;
    }

    Ok(campaign_model)
}

pub async fn get_campaign_by_guid(
    campaign_guid: String,
    db_conn: &DbConn,
) -> Result<Option<database::entity::campaign::Model>, DbErr> {
    let campaign = database::entity::campaign::Entity::find()
        .filter(Condition::any().add(database::entity::campaign::Column::Guid.eq(campaign_guid)))
        .one(db_conn)
        .await?;

    Ok(campaign)
}

pub async fn get_publish_information(
    campaign_guid: String,
    db_conn: &DbConn,
) -> Result<Option<PersistentCampaignDto>, DbErr> {
    let campaign = database::entity::campaign::Entity::find()
        .filter(Condition::any().add(database::entity::campaign::Column::Guid.eq(campaign_guid)))
        .one(db_conn)
        .await?;

    match campaign {
        Some(campaign) => {
            let recipients = database::entity::recipient::Entity::find()
                .filter(
                    Condition::any()
                        .add(database::entity::recipient::Column::CampaignId.eq(campaign.id)),
                )
                .all(db_conn)
                .await?;
            let result = PersistentCampaignDto {
                total_amount: campaign.total_amount.parse().unwrap(),
                number_of_recipients: campaign.number_of_recipients,
                recipients: recipients
                    .into_iter()
                    .map(|x| RecipientDto {
                        address: x.address,
                        amount: x.amount.parse().unwrap(),
                    })
                    .collect(),
            };
            Ok(Some(result))
        }
        None => Ok(None),
    }
}
