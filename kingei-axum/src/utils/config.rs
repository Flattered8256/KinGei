use std::env;

const DEFAULT_MAX_UPLOAD_SIZE_MB: usize = 100;

pub fn max_upload_bytes() -> usize {
    let megabytes = match env::var("MAX_UPLOAD_SIZE_MB") {
        Ok(value) => parse_megabytes(&value).unwrap_or_else(|| {
            panic!("MAX_UPLOAD_SIZE_MB必须是正整数,请检查项目根目录下的.env文件")
        }),
        Err(_) => DEFAULT_MAX_UPLOAD_SIZE_MB,
    };
    megabytes.saturating_mul(1024 * 1024)
}

fn parse_megabytes(value: &str) -> Option<usize> {
    value.trim().parse::<usize>().ok().filter(|mb| *mb > 0)
}

#[cfg(test)]
mod tests {
    use super::parse_megabytes;

    #[test]
    fn parse_positive_megabytes() {
        assert_eq!(parse_megabytes("100"), Some(100));
        assert_eq!(parse_megabytes(" 7 "), Some(7));
        assert_eq!(parse_megabytes("0"), None);
        assert_eq!(parse_megabytes("-1"), None);
        assert_eq!(parse_megabytes("abc"), None);
    }
}
