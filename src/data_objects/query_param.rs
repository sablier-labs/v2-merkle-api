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
