use std::convert::TryInto;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SINParseError {
    TooLong,
    TooShort,
    InvalidChecksum,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SIN {
    inner_digits: [u8; 9]
}

impl SIN {
    pub fn parse(s: String) -> Result<Self, SINParseError> {
        let mut digits = Vec::with_capacity(9);
        for khar in s.chars() {
            if let Some(digit) = khar.to_digit(10) {
                digits.push(digit as u8);
            };
        };
        match digits.len() {
            n if n < 9 => return Err(SINParseError::TooShort),
            n if n > 9 => return Err(SINParseError::TooLong),
            9 => {
                // luhn checksum
                let luhn_sum: u8 = digits.iter()
                    .enumerate()
                    .map(|(idx, digit)| digit * (if idx % 2 == 0 { 1u8 } else { 2u8 }))
                    .map(|val| if val > 9 {
                        // since 16 turns into 1 + 6, and the max value we will se here is 18,
                        // this will always give the right value
                        (val % 10) + 1
                    } else {
                        val
                    })
                    .sum();
                if dbg!(luhn_sum) % 10 != 0 {
                    return Err(SINParseError::InvalidChecksum);
                }
            },
            _ => unreachable!(),
        };
        let boxed_digits = digits.into_boxed_slice();
        let boxing_result: Result<Box<[u8; 9]>, _> = boxed_digits.try_into();
        match boxing_result {
            Ok(val) => Ok(Self { inner_digits: *val }),
            Err(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sin_checks_luhn() {
        assert_eq!(SIN::parse("123456789".to_string()), Err(SINParseError::InvalidChecksum));
        assert_eq!(SIN::parse("425453457".to_string()), Err(SINParseError::InvalidChecksum));
        assert_eq!(SIN::parse("759268676".to_string()), Err(SINParseError::InvalidChecksum));
        assert_eq!(SIN::parse("635563453".to_string()), Err(SINParseError::InvalidChecksum));
        // make sure this doesn't cause an overflow
        assert_eq!(SIN::parse("999999999".to_string()), Err(SINParseError::InvalidChecksum));
        assert!(SIN::parse("046454286".to_string()).is_ok());
        assert!(SIN::parse("000000000".to_string()).is_ok());
    }

    #[test]
    fn parse_sin_checks_too_short() {
        assert_eq!(SIN::parse("12345678".to_string()), Err(SINParseError::TooShort));
        assert_eq!(SIN::parse("123".to_string()), Err(SINParseError::TooShort));
        assert_eq!(SIN::parse("0".to_string()), Err(SINParseError::TooShort));
        assert_eq!(SIN::parse("".to_string()), Err(SINParseError::TooShort));
    }

    #[test]
    fn parse_sin_checks_too_long() {
        assert_eq!(SIN::parse("0000000000".to_string()), Err(SINParseError::TooLong));
        assert_eq!(SIN::parse("4324234237".to_string()), Err(SINParseError::TooLong));
        assert_eq!(SIN::parse("635462452452344343".to_string()), Err(SINParseError::TooLong));
        assert_eq!(SIN::parse("999999999999999999999999999".to_string()), Err(SINParseError::TooLong));
        assert_eq!(SIN::parse("000000000000000000000000000".to_string()), Err(SINParseError::TooLong));
        assert_eq!(SIN::parse("543537672346234345464254235".to_string()), Err(SINParseError::TooLong));
    }
}
