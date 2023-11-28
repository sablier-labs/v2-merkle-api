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
            return Some(ValidationError { row: row_index + 2, message: String::from("Invalid Ethereum address") });
        }
        None
    }

    fn validate_header(&self, cel: &str) -> Option<ValidationError> {
        if cel.to_lowercase() != "address" {
            return Some(ValidationError {
                row: 1, // Header is in the first row
                message: String::from("CSV header invalid. The csv header should be address,amount"),
            });
        }
        None
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
            return Some(ValidationError { row: row_index + 2, message: String::from("The amount cannot be 0") });
        }
        None
    }

    fn validate_header(&self, cel: &str) -> Option<ValidationError> {
        if cel.to_lowercase() != "amount" {
            return Some(ValidationError {
                row: 1, // Header is in the first row
                message: String::from("CSV header invalid. The csv header should be address,amount"),
            });
        }
        None
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

pub fn validate_csv_header(header: &StringRecord, validators: &[&dyn ColumnValidator]) -> Option<ValidationError> {
    for (index, validator) in validators.iter().enumerate() {
        let head = header[index].trim();
        let header_error = validator.validate_header(head);
        if let Some(error) = header_error {
            return Some(error);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_ETH_ADDRESS: &str = "0xf31b00e025584486f7c37Cf0AE0073c97c12c634";
    const INVALID_ETH_ADDRESS: &str = "0xthisIsNotAnAddress";
    const ADDRESS_VALIDATOR: AddressColumnValidator = AddressColumnValidator;

    const AMOUNT_PATTERN: &str = r"^[+]?\d*\.?\d{0,3}$";
    fn get_amount_validator() -> AmountColumnValidator {
        let amount_regex = Regex::new(AMOUNT_PATTERN).unwrap();
        AmountColumnValidator { regex: amount_regex }
    }

    #[test]
    fn is_valid_eth_address_valid() {
        let result = is_valid_eth_address(VALID_ETH_ADDRESS);
        assert!(result);
    }

    #[test]
    fn is_valid_eth_address_invalid() {
        let result = is_valid_eth_address(INVALID_ETH_ADDRESS);
        assert!(!result);
    }

    #[test]
    fn address_column_validator_cel_valid() {
        assert!(ADDRESS_VALIDATOR.validate_cel(VALID_ETH_ADDRESS, 0).is_none());
    }

    #[test]
    fn address_column_validator_cel_invalid() {
        assert!(ADDRESS_VALIDATOR.validate_cel(INVALID_ETH_ADDRESS, 0).is_some());
    }

    #[test]
    fn address_column_validator_header_valid() {
        assert!(ADDRESS_VALIDATOR.validate_header("address").is_none());
    }

    #[test]
    fn address_column_validator_header_invalid() {
        assert!(ADDRESS_VALIDATOR.validate_header("amount").is_some());
    }

    #[test]
    fn amount_column_validator_cel_valid() {
        let validator = get_amount_validator();
        assert!(validator.validate_cel("123.45", 0).is_none());
    }

    #[test]
    fn amount_column_validator_cel_not_a_number() {
        let validator = get_amount_validator();
        assert!(validator.validate_cel("thisIsNotANumber", 0).is_some());
    }

    #[test]
    fn amount_column_validator_cel_zero_() {
        let validator = get_amount_validator();
        assert!(validator.validate_cel("0.0", 0).is_some());
    }

    #[test]
    fn amount_column_validator_header_valid() {
        let validator = get_amount_validator();
        assert!(validator.validate_header("amount").is_none());
    }

    #[test]
    fn amount_column_validator_header_invalid() {
        let validator = get_amount_validator();
        assert!(validator.validate_header("address").is_some());
    }

    #[test]
    fn validate_csv_row_valid() {
        let row = StringRecord::from(vec![VALID_ETH_ADDRESS, "489.312"]);
        let amount_validator = get_amount_validator();
        let validators: Vec<&dyn ColumnValidator> = vec![&AddressColumnValidator, &amount_validator];
        assert!(validate_csv_row(&row, 0, &validators).is_empty());
    }

    #[test]
    fn validate_csv_row_insufficient_columns() {
        let row = StringRecord::from(vec![VALID_ETH_ADDRESS]);
        let amount_validator = get_amount_validator();
        let validators: Vec<&dyn ColumnValidator> = vec![&AddressColumnValidator, &amount_validator];
        assert!(!validate_csv_row(&row, 0, &validators).is_empty());
    }

    #[test]
    fn validate_csv_row_invalid_address() {
        let row = StringRecord::from(vec!["thisIsNotAnAddress", "12534"]);
        let amount_validator = get_amount_validator();
        let validators: Vec<&dyn ColumnValidator> = vec![&AddressColumnValidator, &amount_validator];
        assert!(!validate_csv_row(&row, 0, &validators).is_empty());
    }
    #[test]
    fn validate_csv_row_invalid_amount() {
        let row = StringRecord::from(vec![VALID_ETH_ADDRESS, "12.576757"]);
        let amount_validator = get_amount_validator();
        let validators: Vec<&dyn ColumnValidator> = vec![&AddressColumnValidator, &amount_validator];
        assert!(!validate_csv_row(&row, 0, &validators).is_empty());
    }

    #[test]
    fn validate_csv_header_valid() {
        let header = StringRecord::from(vec!["address", "amount"]);
        let amount_validator = get_amount_validator();
        let validators: Vec<&dyn ColumnValidator> = vec![&AddressColumnValidator, &amount_validator];
        assert!(validate_csv_header(&header, &validators).is_none());
    }

    #[test]
    fn validate_csv_header_invalid_address_header() {
        let header = StringRecord::from(vec!["address_invalid", "amount"]);
        let amount_validator = get_amount_validator();
        let validators: Vec<&dyn ColumnValidator> = vec![&AddressColumnValidator, &amount_validator];
        assert!(validate_csv_header(&header, &validators).is_some());
    }

    #[test]
    fn validate_csv_header_invalid_amount_header() {
        let header = StringRecord::from(vec!["address", "amount_invalid"]);
        let amount_validator = get_amount_validator();
        let validators: Vec<&dyn ColumnValidator> = vec![&AddressColumnValidator, &amount_validator];
        assert!(validate_csv_header(&header, &validators).is_some());
    }
}
