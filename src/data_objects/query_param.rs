use serde::Deserialize;

/// Query parameters for eligibility endpoint
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

/// Query parameters for create endpoint
#[derive(Deserialize)]
pub struct Create {
    #[serde(default = "default_string")]
    pub decimals: String,
}

/// Query parameters for validity endpoint
#[derive(Deserialize)]
pub struct Validity {
    #[serde(default = "default_string")]
    pub cid: String,
}
