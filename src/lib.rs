//! A library for parsing Canadian social insurance numbers and business numbers.

use std::{convert::TryInto, fmt};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// An error resulting from parsing a SIN
pub enum SINParseError {
    /// The SIN is longer than 9 digits.
    TooLong,
    /// The SIN is less than 9 digits.
    TooShort,
    /// The SIN has an invalid Luhn checksum.
    InvalidChecksum,
}

/// Types of SINs: All the provinces, plus some other categories.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SINType {
    /// CRA-assigned Individual Tax Numbers, Temporary Tax Numbers and Adoption Tax Numbers. These
    /// are currently only assigned to natural people.
    CRAAssigned,
    TemporaryResident,
    /// Business numbers and SINs share the same namespace.
    BusinessNumber,
    /// Military forces abroad.
    OverseasForces,
    Alberta,
    BritishColumbia,
    Manitoba,
    NewBrunswick,
    NewfoundlandLabrador,
    NorthwestTerritories,
    NovaScotia,
    Nunavut,
    Ontario,
    PrinceEdwardIsland,
    Quebec,
    Saskatchewan,
    Yukon,
}

impl SINType {
    /// Does the SIN repersent someone in a province?
    pub fn is_province(self) -> bool {
        use SINType::*;
        matches!(
            self,
            Alberta
                | BritishColumbia
                | Manitoba
                | NewBrunswick
                | NewfoundlandLabrador
                | NorthwestTerritories
                | NovaScotia
                | Nunavut
                | Ontario
                | PrinceEdwardIsland
                | Quebec
                | Saskatchewan
                | Yukon
        )
    }
    /// Does the SIN repersent a human? Currently only business numbers are assigned to non-humans.
    pub fn is_human(self) -> bool {
        !matches!(self, Self::BusinessNumber)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// A social insurance number.
pub struct SIN {
    inner_digits: [u8; 9],
}

impl SIN {
    /// Parses a SIN from a string.
    ///
    /// ## Examples
    /// ```
    /// use canada_sin::SIN;
    /// assert!(SIN::parse("046454286".to_string()).is_ok());
    /// ```
    pub fn parse(s: String) -> Result<Self, SINParseError> {
        let mut digits = Vec::with_capacity(9);
        for khar in s.chars() {
            if let Some(digit) = khar.to_digit(10) {
                digits.push(digit as u8);
            };
        }
        match digits.len() {
            n if n < 9 => return Err(SINParseError::TooShort),
            n if n > 9 => return Err(SINParseError::TooLong),
            9 => {
                // luhn checksum
                let luhn_sum: u8 = digits
                    .iter()
                    .enumerate()
                    .map(|(idx, digit)| digit * (if idx % 2 == 0 { 1u8 } else { 2u8 }))
                    .map(|val| {
                        if val > 9 {
                            // since 16 turns into 1 + 6, and the max value we will se here is 18,
                            // this will always give the right value
                            (val % 10) + 1
                        } else {
                            val
                        }
                    })
                    .sum();
                if luhn_sum % 10 != 0 {
                    return Err(SINParseError::InvalidChecksum);
                }
            }
            _ => unreachable!(),
        };
        let boxed_digits = digits.into_boxed_slice();
        let boxing_result: Result<Box<[u8; 9]>, _> = boxed_digits.try_into();
        match boxing_result {
            Ok(val) => Ok(Self { inner_digits: *val }),
            Err(_) => unreachable!(),
        }
    }
    /// All types the SIN *could* be. This will often be multiple options, since this is based on
    /// the first digit, and we are running out of numbers, so there is some overlap. However, the
    /// following can be determined unambiguously:
    /// - `CRAAssigned` (starts with 0)
    /// - `TemporaryResident` (starts with 9)
    /// - `Quebec` (starts with 2 or 3)
    /// - `BusinessNumber` sometimes (if it starts with 8 it's a business number, if it starts with 7 it *might* be one)
    ///
    /// The logic is based on [this mapping](https://en.wikipedia.org/wiki/Social_Insurance_Number#Geography).
    ///
    /// ## Examples
    /// ```
    /// use canada_sin::{SIN, SINType};
    /// let sin = SIN::parse("346454382".to_string()).unwrap();
    /// assert_eq!(sin.types(), vec![SINType::Quebec]);
    /// ```
    ///
    /// ```
    /// use canada_sin::{SIN, SINType::*};
    /// let sin = SIN::parse("734323843".to_string()).unwrap();
    /// assert_eq!(sin.types(), vec![BritishColumbia, Yukon, BusinessNumber]);
    /// ```
    pub fn types(&self) -> Vec<SINType> {
        use SINType::*;
        match self.inner_digits[0] {
            0 => vec![CRAAssigned],
            1 => vec![
                NovaScotia,
                NewBrunswick,
                PrinceEdwardIsland,
                NewfoundlandLabrador,
            ],
            2 | 3 => vec![Quebec],
            4 | 5 => vec![Ontario, OverseasForces],
            6 => vec![
                Ontario,
                Manitoba,
                Saskatchewan,
                Alberta,
                NorthwestTerritories,
                Nunavut,
            ],
            7 => vec![BritishColumbia, Yukon, BusinessNumber],
            8 => vec![BusinessNumber],
            9 => vec![TemporaryResident],
            _ => unreachable!(),
        }
    }
    /// Returns the parsed digits as an array of digits.
    pub fn digits(self) -> [u8; 9] {
        self.inner_digits
    }
    fn gen_sin_string_part(part: &[u8]) -> String {
        part.iter().map(|d| d.to_string()).collect::<String>()
    }
    /// Returns the SIN as a string.
    ///
    /// ## Examples
    /// ```
    /// use canada_sin::SIN;
    /// let sin = SIN::parse("046454286".to_string()).unwrap();
    /// assert_eq!(sin.digits_string(), "046454286")
    /// ```
    pub fn digits_string(self) -> String {
        Self::gen_sin_string_part(&self.inner_digits)
    }
    /// Returns the SIN as a string with dashes in it.
    /// ## Examples
    /// ```
    /// use canada_sin::SIN;
    /// let sin = SIN::parse("046454286".to_string()).unwrap();
    /// assert_eq!(sin.digits_dashed_string(), "046-454-286")
    /// ```
    pub fn digits_dashed_string(self) -> String {
        format!(
            "{}-{}-{}",
            Self::gen_sin_string_part(&self.inner_digits[0..3]),
            Self::gen_sin_string_part(&self.inner_digits[3..6]),
            Self::gen_sin_string_part(&self.inner_digits[6..9]),
        )
    }
}

impl fmt::Display for SIN {
    /// Formats the SIN into three parts with dashes.
    ///
    /// ## Examples
    /// ```
    /// use canada_sin::SIN;
    /// assert_eq!(
    ///     format!("Your SIN is {}.", SIN::parse("046454286".to_string()).unwrap()),
    ///     "Your SIN is 046-454-286.".to_string(),
    /// );
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.digits_dashed_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sin_checks_luhn() {
        assert_eq!(
            SIN::parse("123456789".to_string()),
            Err(SINParseError::InvalidChecksum)
        );
        assert_eq!(
            SIN::parse("425453457".to_string()),
            Err(SINParseError::InvalidChecksum)
        );
        assert_eq!(
            SIN::parse("759268676".to_string()),
            Err(SINParseError::InvalidChecksum)
        );
        assert_eq!(
            SIN::parse("635563453".to_string()),
            Err(SINParseError::InvalidChecksum)
        );
        // make sure this doesn't cause an overflow
        assert_eq!(
            SIN::parse("999999999".to_string()),
            Err(SINParseError::InvalidChecksum)
        );
        assert!(SIN::parse("046454286".to_string()).is_ok());
        assert!(SIN::parse("000000000".to_string()).is_ok());
    }

    #[test]
    fn parse_sin_checks_too_short() {
        assert_eq!(
            SIN::parse("12345678".to_string()),
            Err(SINParseError::TooShort)
        );
        assert_eq!(SIN::parse("123".to_string()), Err(SINParseError::TooShort));
        assert_eq!(SIN::parse("0".to_string()), Err(SINParseError::TooShort));
        assert_eq!(SIN::parse("".to_string()), Err(SINParseError::TooShort));
    }

    #[test]
    fn parse_sin_checks_too_long() {
        assert_eq!(
            SIN::parse("0000000000".to_string()),
            Err(SINParseError::TooLong)
        );
        assert_eq!(
            SIN::parse("4324234237".to_string()),
            Err(SINParseError::TooLong)
        );
        assert_eq!(
            SIN::parse("635462452452344343".to_string()),
            Err(SINParseError::TooLong)
        );
        assert_eq!(
            SIN::parse("999999999999999999999999999".to_string()),
            Err(SINParseError::TooLong)
        );
        assert_eq!(
            SIN::parse("000000000000000000000000000".to_string()),
            Err(SINParseError::TooLong)
        );
        assert_eq!(
            SIN::parse("543537672346234345464254235".to_string()),
            Err(SINParseError::TooLong)
        );
    }

    #[test]
    fn digits_string() {
        let sin = SIN::parse("000-000-000".to_string()).unwrap();
        assert_eq!(sin.digits_string(), "000000000");
        let sin = SIN::parse("999999998".to_string()).unwrap();
        assert_eq!(sin.digits_string(), "999999998");
    }

    #[test]
    fn digits_dashed_string() {
        let sin = SIN::parse("000-000-000".to_string()).unwrap();
        assert_eq!(sin.digits_dashed_string(), "000-000-000");
        let sin = SIN::parse("999999998".to_string()).unwrap();
        assert_eq!(sin.digits_dashed_string(), "999-999-998");
    }
}
