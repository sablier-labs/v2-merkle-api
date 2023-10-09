use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page_number")]
    pub page_number: u64,

    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page_number() -> u64 {
    1
}

fn default_page_size() -> u64 {
    2
}

#[derive(Deserialize)]
pub struct Eligibility {
    #[serde(default = "default_string")]
    pub address: String,

    #[serde(default = "default_string")]
    pub cid: String,
}

fn default_string() -> String {
    "".to_string()
}

#[derive(Deserialize)]
pub struct Create {
    pub decimals: usize,
}