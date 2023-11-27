use serde::Deserialize;

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
    #[serde(default = "default_string")]
    pub decimals: String,
}

#[derive(Deserialize)]
pub struct Validity {
    #[serde(default = "default_string")]
    pub cid: String,
}
