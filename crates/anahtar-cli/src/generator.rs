use crate::config::validate_generator_length;
use anyhow::{Context, Result};
use rand::{rngs::OsRng, seq::SliceRandom};

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{};:,.?/";

pub fn generate_password(length: usize) -> Result<String> {
    validate_generator_length(length)?;
    let classes = [LOWER, UPPER, DIGITS, SYMBOLS];
    let all = classes.concat();
    let mut rng = OsRng;
    let mut bytes = Vec::with_capacity(length);
    for class in classes {
        let ch = class
            .choose(&mut rng)
            .copied()
            .context("password character class is empty")?;
        bytes.push(ch);
    }
    while bytes.len() < length {
        let ch = all
            .choose(&mut rng)
            .copied()
            .context("password character set is empty")?;
        bytes.push(ch);
    }
    bytes.shuffle(&mut rng);
    String::from_utf8(bytes).context("password character set must be ASCII")
}

#[cfg(test)]
mod tests {
    use super::{generate_password, DIGITS, LOWER, SYMBOLS, UPPER};

    #[test]
    fn generated_password_has_requested_length_and_all_character_classes() {
        let password = generate_password(32).unwrap();
        assert_eq!(password.len(), 32);
        assert!(password.bytes().any(|b| LOWER.contains(&b)));
        assert!(password.bytes().any(|b| UPPER.contains(&b)));
        assert!(password.bytes().any(|b| DIGITS.contains(&b)));
        assert!(password.bytes().any(|b| SYMBOLS.contains(&b)));
    }

    #[test]
    fn generated_password_rejects_invalid_length() {
        assert!(generate_password(7).is_err());
        assert!(generate_password(257).is_err());
    }
}
