use sea_orm_migration::prelude::*;

use super::m20220101_000001_create_campaign_table::Campaign;


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Recipient::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Recipient::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Recipient::Address).string().not_null())
                    .col(ColumnDef::new(Recipient::Amount).big_integer().not_null())
                    .col(ColumnDef::new(Recipient::CampaignId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-recipient-campaign_id")
                            .from(Recipient::Table, Recipient::CampaignId)
                            .to(Campaign::Table, Campaign::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Recipient::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Recipient {
    Table,
    Id,
    Address,
    Amount,
    CampaignId,
}
