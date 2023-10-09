use csv::StringRecord;
use ethers_rs::Address;
use regex::Regex;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ValidationError {
    pub row: usize,
    pub message: String,
}

pub fn is_valid_eth_address(address: &str) -> bool {
    Address::try_from(address).is_ok()
}

pub trait ColumnValidator {
    fn validate_cel(&self, cel: &str, row_index: usize) -> Option<ValidationError>;
    fn validate_header(&self, cel: &str) -> Option<ValidationError>;
}

pub struct AddressColumnValidator;
impl ColumnValidator for AddressColumnValidator {
    fn validate_cel(&self, cel: &str, row_index: usize) -> Option<ValidationError> {
        let is_valid = is_valid_eth_address(cel);
        if !is_valid {
            return Some(ValidationError {
                row: row_index + 2,
                message: String::from("Invalid Ethereum address"),
            });
        }
        return None;
    }

    fn validate_header(&self, cel: &str) -> Option<ValidationError> {
        if cel.to_lowercase() != "address" {
            return Some(ValidationError {
                row: 1, // Header is in the first row
                message: String::from(
                    "CSV header invalid. The csv header should be address,amount",
                ),
            });
        }
        return None;
    }
}

pub struct AmountColumnValidator {
    pub regex: Regex,
}

impl ColumnValidator for AmountColumnValidator {
    fn validate_cel(&self, cel: &str, row_index: usize) -> Option<ValidationError> {
        let is_valid = self.regex.is_match(cel);
        if !is_valid {
            return Some(ValidationError {
                row: row_index + 2,
                message: String::from("Amounts should be positive, in normal notation, with an optional decimal point and a maximum number of decimals as provided by the query parameter."),
            });
        }

        let amount: f64 = cel.parse().unwrap();

        if amount == 0.0 {
            return Some(ValidationError {
                row: row_index + 2,
                message: String::from("The amount cannot be 0"),
            });
        }
        return None;
    }

    fn validate_header(&self, cel: &str) -> Option<ValidationError> {
        if cel.to_lowercase() != "amount" {
            return Some(ValidationError {
                row: 1, // Header is in the first row
                message: String::from(
                    "CSV header invalid. The csv header should be address,amount",
                ),
            });
        }
        return None;
    }
}

pub fn validate_csv_row(
    row: &StringRecord,
    row_index: usize,
    validators: &[&dyn ColumnValidator],
) -> Vec<ValidationError> {
    let mut errors: Vec<ValidationError> = Vec::new();
    if row.len() < validators.len() {
        errors.push(ValidationError {
            row: row_index + 2, // +2 to account for CSV header
            message: String::from("Insufficient columns"),
        });
        return errors;
    }
    for (index, validator) in validators.iter().enumerate() {
        let cel = row[index].trim();
        let cel_error = validator.validate_cel(cel, row_index);
        if let Some(error) = cel_error {
            errors.push(error);
        }
    }
    errors
}

pub fn validate_csv_header(
    header: &StringRecord,
    validators: &[&dyn ColumnValidator],
) -> Option<ValidationError> {
    for (index, validator) in validators.iter().enumerate() {
        let head = header[index].trim();
        let header_error = validator.validate_header(head);
        if let Some(error) = header_error {
            return Some(error);
        }
    }
    None
}
