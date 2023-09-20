use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct CampaignDto {
    pub created_at: String,
    pub gid: String,
}

#[derive(Serialize, Debug)]
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
