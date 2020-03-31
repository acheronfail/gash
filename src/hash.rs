use std::ops::Range;

fn hex_to_range(ch: &char) -> Range<u8> {
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

// TODO: return a function so prefix is cached
pub fn create_validator<T: AsRef<[u8]>>(prefix: &str) -> impl Fn(T) -> bool + '_ {
  let (hash_start, hash_last_range) = if prefix.len() % 2 == 0 {
    (hex::decode(prefix).unwrap(), None)
  } else {
    let ch = prefix
      .chars()
      .skip(prefix.len() - 1)
      .take(1)
      .next()
      .unwrap();

    (
      hex::decode(&prefix[..prefix.len() - 1]).unwrap(),
      Some(hex_to_range(&ch)),
    )
  };

  move |h| {
    let hash = h.as_ref();
    if hash.starts_with(&hash_start) {
      return match &hash_last_range {
        None => true,
        Some(range) => range.contains(&hash[(prefix.len() - 1) / 2]),
      };
    }

    false
  }
}

#[cfg(test)]
mod tests {
  use crate::hash::create_validator;

  #[test]
  fn it_checks_hashes_correctly() {
    let t = |p, h| create_validator(p)(hex::decode(h).unwrap());

    assert_eq!(true, t("abcd", "abcd000000"));
    assert_eq!(true, t("abc", "abcd000000"));
    assert_eq!(true, t("ab", "abcd000000"));
    assert_eq!(true, t("a", "abcd000000"));
    assert_eq!(true, t("", "abcd000000"));

    assert_eq!(false, t("abcd", "deadbeef00"));
    assert_eq!(false, t("abc", "deadbeef00"));
    assert_eq!(false, t("ab", "deadbeef00"));
    assert_eq!(false, t("a", "deadbeef00"));
  }
}
