use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct CampaignDto {
    pub created_at: String,
    pub total_amount: f64,
    pub number_of_recipients: i32,
    pub guid: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipientDto {
    pub address: String,
    pub amount: f64,
}

#[derive(Serialize, Debug)]
pub struct RecipientPageDto {
    pub page_number: u64,
    pub page_size: u64,
    pub recipients: Vec<RecipientDto>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PersistentCampaignDto {
    pub total_amount: f64,
    pub number_of_recipients: i32,
    pub recipients: Vec<RecipientDto>,
}
