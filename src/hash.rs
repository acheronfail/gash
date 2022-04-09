use std::ops::RangeInclusive;

use anyhow::Result;

/// Converts a hex character into a u8 range.
/// Hex-encoded strings contain two characters for each u8 byte. When given a
/// hex character, this function assumes it is the first of a pair of hex characters
/// and returns the range of possible byte values it might be.
fn word_range_for_char(ch: char) -> RangeInclusive<u8> {
    match ch {
        '0' => 0..=15,
        '1' => 16..=31,
        '2' => 32..=47,
        '3' => 48..=63,
        '4' => 64..=79,
        '5' => 80..=95,
        '6' => 96..=111,
        '7' => 112..=127,
        '8' => 128..=143,
        '9' => 144..=159,
        'a' => 160..=175,
        'b' => 176..=191,
        'c' => 192..=207,
        'd' => 208..=223,
        'e' => 224..=239,
        'f' => 240..=255,
        _ => panic!("not meant to happen"),
    }
}

// Returns a function that validates a hash with the given signature.
// Hex hashes contain twice as many characters as bytes (each byte is split into two characters).
// So, we change validation strategies if the signature is even or odd.
// (And also if it's a prefix or a suffix.)
pub fn create_validator<T: AsRef<[u8]>>(
    sig: &str,
    as_suffix: bool,
) -> Result<impl Fn(T) -> bool + '_> {
    let is_even = sig.len() % 2 == 0;
    let (sig_even_chunk, odd_char) = if is_even {
        // For signatures of even length, we convert the entire signature to bytes.
        (hex::decode(sig)?, None)
    } else {
        // For signatures of odd length extract the first/last character:
        let ch = sig
            .chars()
            // When using a suffix, we extract the first char: xxxxxxSig
            // When using a prefix, it's the last char:        siGxxxxxx
            .skip(if as_suffix { 0 } else { sig.len() - 1 })
            .take(1)
            .next()
            // SAFETY: we limit signature length to 40 (length of a sha1 hash)
            .unwrap();

        let even_chunk = if as_suffix {
            &sig[1..]
        } else {
            &sig[..sig.len() - 1]
        };
        (
            // And convert all but the odd character (the even portion) to bytes.
            hex::decode(&even_chunk)?,
            Some(ch),
        )
    };

    Ok(move |h: T| {
        let hash = h.as_ref();

        // Does the hash start/end with the even portion of our signature?
        if as_suffix {
            if !hash.ends_with(&sig_even_chunk) {
                return false;
            }
        } else {
            if !hash.starts_with(&sig_even_chunk) {
                return false;
            }
        }

        if is_even {
            // If the signature was even, then we've checked the whole hash already.
            true
        } else {
            // SAFETY: this is always set if `as_suffix == true`
            let odd_char = odd_char.unwrap();

            // If it was odd, then our even signature matched, but we also need
            // to check that the leading/trailing word will produce our odd character
            if as_suffix {
                // Check if the word ends with the right character
                let pos = hash.len() - ((sig.len() - 1) / 2) - 1;
                let word = &hash[pos];
                let byte = odd_char
                    .to_digit(16)
                    // SAFETY: we validate that the signature only contains valid
                    // hex characters, which are a subset of ASCII
                    .unwrap() as u8;

                *word >= byte && (word - byte) & 0xf == 0
            } else {
                // Check if the word starts with the right character
                let word = &hash[(sig.len() - 1) / 2];
                let allowed_range = word_range_for_char(odd_char);
                allowed_range.contains(word)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::hash::{create_validator, word_range_for_char};

    #[test]
    fn word_ranges() {
        for i in 0x00..=0xff_u8 {
            let ch = char::from_digit((i / 16).into(), 16).unwrap();
            println!("{} = {}", ch, i);
            assert_eq!(true, word_range_for_char(ch).contains(&i));
        }
    }

    // Validator edge cases

    #[test]
    fn validator_odd_prefix() {
        // Odd signature + prefix uses range to check validity
        let validator = create_validator("dad", false).unwrap();
        for ch in 0x0..=0xf_u8 {
            let ch = char::from_digit(ch.into(), 16).unwrap();
            let hash_bytes = hex::decode(format!("da{}0", ch)).unwrap();
            assert_eq!(matches!(ch, 'd'), validator(hash_bytes));
        }
    }

    #[test]
    fn validator_odd_suffix() {
        // Odd signature + suffix uses subtraction to test validity
        let validator = create_validator("dad", true).unwrap();
        for ch in 0x0..=0xf_u8 {
            let ch = char::from_digit(ch.into(), 16).unwrap();
            let hash_bytes = hex::decode(format!("0{}ad", ch)).unwrap();
            assert_eq!(matches!(ch, 'd'), validator(hash_bytes));
        }
    }

    // Test signature validation

    fn test_sig(sig: &str, hash: &str, suffix: bool) -> bool {
        let validator = create_validator(sig, suffix).unwrap();
        let hash_bytes = hex::decode(hash).unwrap();
        validator(hash_bytes)
    }

    #[test]
    fn boundary_length_conditions() {
        // Even, prefix (full length)
        assert_eq!(true, test_sig("123456", "123456", false));
        // Even, suffix (full length)
        assert_eq!(true, test_sig("123456", "123456", true));
        // Odd, prefix  (length - 1)
        assert_eq!(true, test_sig("12345", "12345f", false));
        // Odd, suffix  (length - 1)
        assert_eq!(true, test_sig("12345", "f12345", true));
    }

    #[test]
    fn returns_true_if_hash_starts_with_prefix() {
        assert_eq!(true, test_sig("abcd", "abcd123456", false));
        assert_eq!(true, test_sig("abc", "abcd123456", false));
        assert_eq!(true, test_sig("ab", "abcd123456", false));
        assert_eq!(true, test_sig("a", "abcd123456", false));
    }

    #[test]
    fn returns_true_with_empty_prefix() {
        assert_eq!(true, test_sig("", "abcd123456", false));
    }

    #[test]
    fn returns_false_if_hash_does_not_start_with_prefix() {
        assert_eq!(false, test_sig("abcd", "deadbeef00", false));
        assert_eq!(false, test_sig("abc", "deadbeef00", false));
        assert_eq!(false, test_sig("ab", "deadbeef00", false));
        assert_eq!(false, test_sig("a", "deadbeef00", false));
    }

    #[test]
    fn returns_true_if_hash_ends_with_suffix() {
        assert_eq!(true, test_sig("abcd", "123456abcd", true));
        assert_eq!(true, test_sig("bcd", "123456abcd", true));
        assert_eq!(true, test_sig("cd", "123456abcd", true));
        assert_eq!(true, test_sig("d", "123456abcd", true));
    }

    #[test]
    fn returns_true_with_empty_suffix() {
        assert_eq!(true, test_sig("", "abcd123456", true));
    }

    #[test]
    fn returns_false_if_hash_does_not_end_with_suffix() {
        assert_eq!(false, test_sig("abcd", "00deadbeef", false));
        assert_eq!(false, test_sig("bcd", "00deadbeef", false));
        assert_eq!(false, test_sig("cd", "00deadbeef", false));
        assert_eq!(false, test_sig("d", "00deadbeef", false));
    }
}
