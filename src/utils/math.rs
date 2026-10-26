use std::str::FromStr;
use substreams::scalar::BigDecimal;

pub fn decimal_from_str(value: &str) -> Result<BigDecimal, bigdecimal::ParseBigDecimalError> {
    BigDecimal::from_str(value)
}

/// Exact base-10 shift: one wei remains 0.000000000000000001 ETH.
pub fn convert_and_divide(value: &str) -> Result<BigDecimal, bigdecimal::ParseBigDecimalError> {
    Ok(BigDecimal::from_str(value)? / BigDecimal::from(1_000_000_000_000_000_000u64))
}
