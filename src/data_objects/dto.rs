use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RecipientDto {
    pub address: String,
    pub amount: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PersistentCampaignDto {
    pub total_amount: String,
    pub number_of_recipients: i32,
    pub root: String,
    pub merkle_tree: String,
    pub recipients: Vec<RecipientDto>,
}
