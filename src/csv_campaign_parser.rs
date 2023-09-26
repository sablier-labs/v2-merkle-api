use csv::Reader;
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::error::Error;

#[derive(Serialize, Debug)]
pub struct ValidationError {
    pub row: usize,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CampaignCsvRecord {
    pub address: String,
    pub amount: f64,
}

pub struct CampaignCsvParsed {
    pub records: Vec<CampaignCsvRecord>,
    pub validation_errors: Vec<ValidationError>,
}

impl CampaignCsvParsed {
    pub fn build(rdr: Reader<&[u8]>) -> Result<CampaignCsvParsed, Box<dyn Error>> {
        let mut rdr = rdr;
        let address_regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
        let positive_number_regex = Regex::new(r"^[+]?\d*\.?\d+$").unwrap();
        let mut validation_errors = Vec::new();
        let mut records: Vec<CampaignCsvRecord> = Vec::new();

        // Validate the CSV header
        let header = rdr.headers()?;
        if !(header.get(0) == Some("address")) || !(header.get(1) == Some("amount")) {
            validation_errors.push(ValidationError {
                row: 1, // Header is in the first row
                message: String::from(
                    "CSV header invalid. The csv header should be address,amount",
                ),
            });
        }

        for (row_index, result) in rdr.records().enumerate() {
            let record = result?;

            if validation_errors.len() >= 100 {
                break;
            }

            if record.len() < 2 {
                validation_errors.push(ValidationError {
                    row: row_index + 2, // +2 to account for CSV header
                    message: String::from("Insufficient columns"),
                });
                continue;
            }

            let address_field = record[0].trim();
            let amount_field = record[1].trim();

            if !address_regex.is_match(address_field) {
                validation_errors.push(ValidationError {
                    row: row_index + 2,
                    message: String::from("Invalid Ethereum address"),
                });
            }

            if !positive_number_regex.is_match(amount_field) {
                validation_errors.push(ValidationError {
                    row: row_index + 2,
                    message: String::from("Invalid amount. Amount should be a positive number"),
                });
            }

            if validation_errors.len() == 0 {
                let parsed_rec = CampaignCsvRecord {
                    address: address_field.to_string(),
                    amount: amount_field.parse().unwrap(),
                };
                records.push(parsed_rec);
            }
        }

        Ok(CampaignCsvParsed {
            records,
            validation_errors,
        })
    }
}

impl CampaignCsvRecord {
    pub fn to_bytes(&self) -> Vec<u8> {
        format!("{}{}", self.address, self.amount).into_bytes()
    }

    pub fn to_hashed_bytes(&self) -> [u8; 32] {
        let hashed = Sha256::digest(&self.to_bytes());
        let mut array = [0u8; 32];
        array.copy_from_slice(&hashed);
        array
    }
}
