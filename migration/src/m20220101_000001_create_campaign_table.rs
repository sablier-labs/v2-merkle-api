use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Campaign::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Campaign::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Campaign::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Campaign::Gid).uuid().not_null().unique_key())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Campaign::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Campaign {
    Table,
    Id,
    CreatedAt,
    Gid,
}
