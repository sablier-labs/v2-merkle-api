use csv::Reader;
use regex::Regex;
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, error::Error};

use crate::utils::csv_validator::{
    validate_csv_header, validate_csv_row, AddressColumnValidator, AmountColumnValidator,
    ColumnValidator, ValidationError,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CampaignCsvRecord {
    pub address: String,
    pub amount: u128,
}

pub struct CampaignCsvParsed {
    pub records: Vec<CampaignCsvRecord>,
    pub validation_errors: Vec<ValidationError>,
    pub number_of_recipients: i32,
    pub total_amount: u128,
}

impl CampaignCsvParsed {
    pub fn build(rdr: Reader<&[u8]>, decimals: usize) -> Result<CampaignCsvParsed, Box<dyn Error>> {
        println!("Validation start: {:?}", std::time::SystemTime::now());
        let mut rdr = rdr;
        let mut validation_errors = Vec::new();
        let mut records: Vec<CampaignCsvRecord> = Vec::new();
        let mut total_amount: u128 = 0;
        let mut number_of_recipients: i32 = 0;
        let pattern = format!(r"^[+]?\d*\.?\d{{0,{}}}$", decimals);
        let amount_regex = Regex::new(&pattern).unwrap();

        let amount_validator = AmountColumnValidator {
            regex: amount_regex,
        };
        let address_validator = AddressColumnValidator;

        let validators: Vec<&dyn ColumnValidator> = vec![&address_validator, &amount_validator];
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

            let address_field = record[0].trim();
            let amount_field = record[1].trim();
            let row = row_index + 2;
            let row_errors = validate_csv_row(&record, row_index, &validators);
            if row_errors.len() > 0 {
                validation_errors.extend(row_errors);
            }

            if unique_addresses.contains(address_field) {
                validation_errors.push(ValidationError {
                    row,
                    message: String::from("Each recipient should have an unique address. This address was already specified in file"),
                });
            }

            if validation_errors.len() == 0 {
                let padded_amount = pad_value(amount_field, decimals);
                total_amount += padded_amount;
                number_of_recipients += 1;
                unique_addresses.insert(address_field.to_string());
                records.push(CampaignCsvRecord {
                    address: address_field.to_string(),
                    amount: padded_amount,
                });
            }
        }

        println!("Validation stop {:?}", std::time::SystemTime::now());

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

fn pad_value(s: &str, no_decimals: usize) -> u128 {
    let decimal_point = s.find('.').unwrap_or_else(|| s.len());
    if decimal_point == s.len() {
        return format!("{}{}", s, "0".repeat(no_decimals)).parse().unwrap();
    }

    let decimals = s.len() - decimal_point - 1;
    let mut result = String::with_capacity(s.len() + no_decimals - decimals);
    result.push_str(&s[0..decimal_point]);
    result.push_str(&s[decimal_point + 1..]);

    for _ in 0..(no_decimals - decimals) {
        result.push('0');
    }

    result.parse().unwrap()
}
