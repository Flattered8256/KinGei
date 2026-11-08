use std::env;

const DEFAULT_MAX_UPLOAD_MB: u64 = 100;

pub fn max_upload_bytes() -> u64 {
    match env::var("MAX_UPLOAD_SIZE_MB") {
        Ok(value) => parse_size_mb(&value).unwrap_or_else(|| {
            panic!("MAX_UPLOAD_SIZE_MB必须是正整数,请检查项目根目录下的.env文件")
        }),
        Err(_) => DEFAULT_MAX_UPLOAD_MB * 1024 * 1024,
    }
}

fn parse_size_mb(value: &str) -> Option<u64> {
    value
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|mb| *mb > 0)
        .and_then(|mb| mb.checked_mul(1024 * 1024))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_positive_size() {
        assert_eq!(parse_size_mb("100"), Some(100 * 1024 * 1024));
        assert_eq!(parse_size_mb(" 1 "), Some(1024 * 1024));
    }

    #[test]
    fn reject_invalid_size() {
        assert_eq!(parse_size_mb("0"), None);
        assert_eq!(parse_size_mb("-5"), None);
        assert_eq!(parse_size_mb("abc"), None);
        assert_eq!(parse_size_mb(""), None);
    }
}
