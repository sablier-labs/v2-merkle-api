use csv::Reader;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, error::Error};

use crate::utils::csv_validator::{
    validate_csv_header, validate_csv_row, AddressColumnValidator, AmountColumnValidator,
    ColumnValidator, ValidationError,
};

#[derive(Clone, Debug, Serialize)]
pub struct CampaignCsvRecord {
    pub address: String,
    pub amount: i64,
}

pub struct CampaignCsvParsed {
    pub records: Vec<CampaignCsvRecord>,
    pub validation_errors: Vec<ValidationError>,
    pub number_of_recipients: usize,
    pub total_amount: i64,
}

impl CampaignCsvParsed {
    pub fn build(rdr: Reader<&[u8]>, decimals: u32) -> Result<CampaignCsvParsed, Box<dyn Error>> {
        let mut rdr = rdr;
        let mut validation_errors = Vec::new();
        let mut records: Vec<CampaignCsvRecord> = Vec::new();
        let mut total_amount: i64 = 0;
        let mut number_of_recipients: usize = 0;
        let validators: Vec<&dyn ColumnValidator> =
            vec![&AddressColumnValidator, &AmountColumnValidator];
        let mut unique_addresses: HashSet<String> = HashSet::new();

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
            let amount: f64 = amount_field.parse().unwrap();

            if unique_addresses.contains(address_field) {
                validation_errors.push(ValidationError {
                    row: row_index + 2,
                    message: String::from("Each recipient should have an unique address. This address was already specified in file"),
                });
            }

            if validation_errors.len() == 0 {
                let padded_amount = amount * (10i64.pow(decimals) as f64);
                total_amount += padded_amount as i64;
                number_of_recipients += 1;
                unique_addresses.insert(address_field.to_string());
                records.push(CampaignCsvRecord {
                    address: address_field.to_string(),
                    amount: padded_amount as i64,
                });
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
