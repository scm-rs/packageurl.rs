mod encodable;
mod quickfind;

pub use self::encodable::PercentCodec;
pub use self::quickfind::QuickFind;
use std::borrow::Cow;

pub fn rcut(input: &str, sep: u8) -> (&str, &str) {
    if let Some(i) = input.quickrfind(sep) {
        (&input[..i], &input[i + 1..])
    } else {
        ("", input)
    }
}

pub fn cut(input: &str, sep: u8) -> (&str, &str) {
    if let Some(i) = input.quickfind(sep) {
        (&input[..i], &input[i + 1..])
    } else {
        (input, "")
    }
}

pub(crate) fn to_lowercase(s: Cow<str>) -> Cow<str> {
    if !s.chars().any(|c| c.is_uppercase()) {
        Cow::Owned(s.to_lowercase())
    } else {
        s
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cut() {
        let buf = "A:B:C";
        assert_eq!(cut(buf, b':'), ("A", "B:C"));
        assert_eq!(cut(buf, b','), ("A:B:C", ""));
    }

    #[test]
    fn test_rcut() {
        let buf = "A:B:C";
        assert_eq!(rcut(buf, b':'), ("A:B", "C"));
        assert_eq!(rcut(buf, b','), ("", "A:B:C"));
    }
}
