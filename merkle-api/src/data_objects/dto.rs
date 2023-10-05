use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct CampaignDto {
    pub total_amount: u128,
    pub number_of_recipients: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipientDto {
    pub address: String,
    pub amount: String,
}

#[derive(Serialize, Debug)]
pub struct RecipientPageDto {
    pub page_number: u64,
    pub page_size: u64,
    pub recipients: Vec<RecipientDto>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PersistentCampaignDto {
    pub total_amount: String,
    pub number_of_recipients: i32,
    pub recipients: Vec<RecipientDto>,
}
