use csv::Reader;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, error::Error};

use crate::utils::csv_validator::{
    validate_csv_header, validate_csv_row, AddressColumnValidator, AmountColumnValidator,
    ColumnValidator, ValidationError,
};

#[derive(Clone, Debug, Serialize)]
pub struct CampaignCsvRecord {
    pub address: String,
    pub amount: f64,
}

pub struct CampaignCsvParsed {
    pub records: HashMap<String, f64>,
    pub validation_errors: Vec<ValidationError>,
    pub number_of_recipients: usize,
    pub total_amount: f64,
}

impl CampaignCsvParsed {
    pub fn build(rdr: Reader<&[u8]>, decimals: u32) -> Result<CampaignCsvParsed, Box<dyn Error>> {
        let mut rdr = rdr;
        let mut validation_errors = Vec::new();
        let mut records: HashMap<String, f64> = HashMap::new();
        let mut total_amount: f64 = 0.0;
        let mut number_of_recipients: usize = 0;
        let validators: Vec<&dyn ColumnValidator> =
            vec![&AddressColumnValidator, &AmountColumnValidator];

        // Validate the CSV header
        let header = rdr.headers()?;
        let header_errors = validate_csv_header(header, &validators);
        if let Some(error) = header_errors {
            validation_errors.push(error);
            return Ok(CampaignCsvParsed {
                total_amount,
                number_of_recipients,
                records,
                validation_errors,
            });
        }

        for (row_index, result) in rdr.records().enumerate() {
            let record = result?;

            if validation_errors.len() >= 100 {
                break;
            }

            let row_errors = validate_csv_row(&record, row_index, decimals, &validators);
            if row_errors.len() > 0 {
                validation_errors.extend(row_errors);
            }

            let address_field = record[0].trim();
            let amount_field = record[1].trim();
            let amount = amount_field.parse().unwrap();

            if records.contains_key(address_field) {
                validation_errors.push(ValidationError {
                    row: row_index + 2,
                    message: String::from("Each recipient should have an unique address. This address was already specified in file"),
                });
            }

            if validation_errors.len() == 0 {
                total_amount += amount;
                number_of_recipients += 1;
                records.insert(address_field.to_string(), amount);
            }
        }

        Ok(CampaignCsvParsed {
            total_amount,
            number_of_recipients,
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
