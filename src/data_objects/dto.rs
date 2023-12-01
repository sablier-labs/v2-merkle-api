use serde::{Deserialize, Serialize};

/// Struct that represents the abstraction of an airstream campaign recipient
#[derive(Deserialize, Serialize, Debug)]
pub struct RecipientDto {
    pub address: String,
    pub amount: String,
}

/// Struct that represents the abstraction of an airstream campaign
#[derive(Deserialize, Serialize, Debug)]
pub struct PersistentCampaignDto {
    pub total_amount: String,
    pub number_of_recipients: i32,
    pub root: String,
    pub merkle_tree: String,
    pub recipients: Vec<RecipientDto>,
}
