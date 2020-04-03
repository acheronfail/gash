use std::ops::Range;

/// Converts a hex character into a u8 range.
/// Hex-encoded strings contain two characters for each u8 byte. When given a
/// hex character, this function assumes it is the first of a pair of hex characters
/// and returns the range of possible byte values it might be.
fn hex_to_range(ch: char) -> Range<u8> {
    match ch {
        '0' => 0..15,
        '1' => 16..31,
        '2' => 32..47,
        '3' => 48..63,
        '4' => 64..79,
        '5' => 80..95,
        '6' => 96..111,
        '7' => 112..127,
        '8' => 128..143,
        '9' => 144..159,
        'a' => 160..175,
        'b' => 176..191,
        'c' => 192..207,
        'd' => 208..223,
        'e' => 224..239,
        'f' => 240..255,
        _ => panic!("not meant to happen"),
    }
}

// Returns a function that validates a hash with the given prefix.
// Hex hashes contain twice as many characters as bytes (each byte is split into two characters).
// So, we change validation strategies if the prefix is even or odd.
pub fn create_validator<T: AsRef<[u8]>>(prefix: &str) -> impl Fn(T) -> bool + '_ {
    // For prefixes of even length, we convert the entire prefix to bytes.
    let (hash_start, hash_last_range) = if prefix.len() % 2 == 0 {
        (hex::decode(prefix).unwrap(), None)
    } else {
        // For prefixes of odd length extract the last character:
        let ch = prefix
            .chars()
            .skip(prefix.len() - 1)
            .take(1)
            .next()
            .unwrap();

        (
            // And convert all but the last character (the even portion) to bytes.
            hex::decode(&prefix[..prefix.len() - 1]).unwrap(),
            // Return a range of what the last byte might be.
            Some(hex_to_range(ch)),
        )
    };

    move |h| {
        let hash = h.as_ref();

        // Does the hash start with the even portion of our prefix?
        if !hash.starts_with(&hash_start) {
            return false;
        }

        match &hash_last_range {
            // If it was odd, then our even prefix matched, but we also need
            // to check that the last byte is within a range that will produce
            // the last odd character.
            Some(range) => range.contains(&hash[(prefix.len() - 1) / 2]),
            // If the prefix was even, then we've checked the whole hash.
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hash::create_validator;

    fn test_prefix(prefix: &str, hash: &str) -> bool {
        create_validator(prefix)(hex::decode(hash).unwrap())
    }

    #[test]
    fn returns_true_if_hash_starts_with_prefix() {
        assert_eq!(true, test_prefix("abcd", "abcd000000"));
        assert_eq!(true, test_prefix("abc", "abcd000000"));
        assert_eq!(true, test_prefix("ab", "abcd000000"));
        assert_eq!(true, test_prefix("a", "abcd000000"));
    }

    #[test]
    fn returns_true_with_empty_prefix() {
        assert_eq!(true, test_prefix("", "abcd000000"));
    }

    #[test]
    fn returns_false_if_hash_does_not_start_with_prefix() {
        assert_eq!(false, test_prefix("abcd", "deadbeef00"));
        assert_eq!(false, test_prefix("abc", "deadbeef00"));
        assert_eq!(false, test_prefix("ab", "deadbeef00"));
        assert_eq!(false, test_prefix("a", "deadbeef00"));
    }
}
