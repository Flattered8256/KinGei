use std::env;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::model::file::MediaKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaProbe {
    pub kind: MediaKind,
    pub width: i64,
    pub height: i64,
    pub duration_ms: Option<i64>,
}

pub const THUMB_MAX: i64 = 320;

const IMAGE_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];
const VIDEO_TYPES: &[&str] = &["video/mp4", "video/quicktime", "video/webm"];

pub fn require_tools() {
    require_bin(
        "FFPROBE_PATH",
        "ffprobe",
        "缺少ffprobe,请安装ffmpeg并确保ffprobe在PATH中,或设置FFPROBE_PATH",
    );
    require_bin(
        "FFMPEG_PATH",
        "ffmpeg",
        "缺少ffmpeg,请安装ffmpeg并确保ffmpeg在PATH中,或设置FFMPEG_PATH",
    );
}

fn require_bin(env_key: &str, default: &str, message: &str) {
    let path = env::var(env_key).unwrap_or_else(|_| default.to_string());
    let output = Command::new(&path).arg("-version").output();
    match output {
        Ok(out) if out.status.success() => {}
        _ => panic!("{message}"),
    }
}

pub fn media_kind_from_content_type(content_type: &str) -> Option<MediaKind> {
    if IMAGE_TYPES
        .iter()
        .any(|allowed| content_type.eq_ignore_ascii_case(allowed))
    {
        Some(MediaKind::Image)
    } else if VIDEO_TYPES
        .iter()
        .any(|allowed| content_type.eq_ignore_ascii_case(allowed))
    {
        Some(MediaKind::Video)
    } else {
        None
    }
}

pub fn thumb_dimensions(width: i64, height: i64) -> (i64, i64) {
    let long = width.max(height);
    if long <= THUMB_MAX {
        return (width.max(1), height.max(1));
    }
    let scale = THUMB_MAX as f64 / long as f64;
    let width = (width as f64 * scale).round().max(1.0) as i64;
    let height = (height as f64 * scale).round().max(1.0) as i64;
    (width, height)
}

pub fn probe_path(path: &Path, kind: MediaKind) -> Option<MediaProbe> {
    let bin = env::var("FFPROBE_PATH").unwrap_or_else(|_| "ffprobe".to_string());
    let output = Command::new(&bin)
        .args([
            "-v",
            "error",
            "-hide_banner",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
        ])
        .arg(path)
        .output();
    let output = match output {
        Ok(out) if out.status.success() => out,
        Ok(out) => {
            tracing::error!(
                status = ?out.status,
                stderr = %String::from_utf8_lossy(&out.stderr),
                "ffprobe failed"
            );
            return None;
        }
        Err(e) => {
            tracing::error!(?e, "ffprobe spawn failed");
            return None;
        }
    };
    parse_probe_json(&output.stdout, kind)
}

pub fn write_thumbnail(src: &Path, dst: &Path, width: i64, height: i64) -> Result<(), String> {
    let (tw, th) = thumb_dimensions(width, height);
    let vf = format!("scale={tw}:{th}");
    let bin = env::var("FFMPEG_PATH").unwrap_or_else(|_| "ffmpeg".to_string());
    let output = Command::new(&bin)
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(src)
        .args(["-frames:v", "1", "-q:v", "4", "-vf", &vf])
        .arg(dst)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    if !dst.is_file() {
        return Err("ffmpeg wrote no thumbnail".to_string());
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ProbeJson {
    streams: Option<Vec<ProbeStream>>,
    format: Option<ProbeFormat>,
}

#[derive(Debug, Deserialize)]
struct ProbeStream {
    codec_type: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    tags: Option<ProbeTags>,
    side_data_list: Option<Vec<SideData>>,
}

#[derive(Debug, Deserialize)]
struct ProbeTags {
    rotate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SideData {
    rotation: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
}

pub fn parse_probe_json(bytes: &[u8], kind: MediaKind) -> Option<MediaProbe> {
    let json: ProbeJson = match serde_json::from_slice(bytes) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!(?e, "ffprobe json");
            return None;
        }
    };
    let stream = json.streams.as_ref()?.iter().find(|stream| {
        stream
            .codec_type
            .as_deref()
            .is_some_and(|codec| codec.eq_ignore_ascii_case("video"))
    })?;
    let mut width = stream.width.filter(|value| *value > 0)?;
    let mut height = stream.height.filter(|value| *value > 0)?;
    let rotate_tag = stream.tags.as_ref().and_then(|tags| tags.rotate.as_deref());
    let side_rotation = stream
        .side_data_list
        .as_ref()
        .and_then(|list| list.iter().find_map(|item| item.rotation));
    if should_swap(rotate_tag, side_rotation) {
        std::mem::swap(&mut width, &mut height);
    }
    let duration_ms = if kind == MediaKind::Video {
        json.format
            .as_ref()
            .and_then(|format| format.duration.as_deref())
            .and_then(|value| value.parse::<f64>().ok())
            .map(|seconds| (seconds * 1000.0).round() as i64)
            .filter(|ms| *ms > 0)
    } else {
        None
    };
    Some(MediaProbe {
        kind,
        width,
        height,
        duration_ms,
    })
}

fn should_swap(rotate_tag: Option<&str>, side_rotation: Option<f64>) -> bool {
    if let Some(tag) = rotate_tag
        && let Ok(deg) = tag.parse::<f64>()
    {
        return is_odd_right_angle(deg);
    }
    side_rotation.is_some_and(is_odd_right_angle)
}

fn is_odd_right_angle(deg: f64) -> bool {
    let n = ((deg % 360.0) + 360.0) % 360.0;
    let quarter = (n / 90.0).round() as i32 % 4;
    quarter == 1 || quarter == 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_image_ignores_duration() {
        let json = br#"{
            "streams": [{"codec_type": "video", "width": 800, "height": 600}],
            "format": {"duration": "1.000"}
        }"#;
        let media = parse_probe_json(json, MediaKind::Image).unwrap();
        assert_eq!(media.kind, MediaKind::Image);
        assert_eq!(media.width, 800);
        assert_eq!(media.height, 600);
        assert_eq!(media.duration_ms, None);
    }

    #[test]
    fn parse_video_duration_ms() {
        let json = br#"{
            "streams": [{"codec_type": "video", "width": 1920, "height": 1080}],
            "format": {"duration": "12.345"}
        }"#;
        let media = parse_probe_json(json, MediaKind::Video).unwrap();
        assert_eq!(media.kind, MediaKind::Video);
        assert_eq!(media.width, 1920);
        assert_eq!(media.height, 1080);
        assert_eq!(media.duration_ms, Some(12345));
    }

    #[test]
    fn parse_rotate_tag_swaps() {
        let json = br#"{
            "streams": [{
                "codec_type": "video",
                "width": 1920,
                "height": 1080,
                "tags": {"rotate": "90"}
            }],
            "format": {"duration": "3"}
        }"#;
        let media = parse_probe_json(json, MediaKind::Video).unwrap();
        assert_eq!(media.width, 1080);
        assert_eq!(media.height, 1920);
    }

    #[test]
    fn parse_displaymatrix_swaps() {
        let json = br#"{
            "streams": [{
                "codec_type": "video",
                "width": 1920,
                "height": 1080,
                "side_data_list": [{"rotation": -90.0}]
            }]
        }"#;
        let media = parse_probe_json(json, MediaKind::Image).unwrap();
        assert_eq!(media.width, 1080);
        assert_eq!(media.height, 1920);
    }

    #[test]
    fn parse_missing_video_stream() {
        let json = br#"{
            "streams": [{"codec_type": "audio", "sample_rate": "44100"}]
        }"#;
        assert!(parse_probe_json(json, MediaKind::Video).is_none());
    }

    #[test]
    fn parse_bad_json() {
        assert!(parse_probe_json(b"not json", MediaKind::Image).is_none());
    }

    #[test]
    fn thumb_fits_inside_max() {
        assert_eq!(thumb_dimensions(1920, 1080), (320, 180));
        assert_eq!(thumb_dimensions(1080, 1920), (180, 320));
        assert_eq!(thumb_dimensions(200, 100), (200, 100));
    }

    #[test]
    fn kind_from_mime() {
        assert_eq!(
            media_kind_from_content_type("image/jpeg"),
            Some(MediaKind::Image)
        );
        assert_eq!(
            media_kind_from_content_type("VIDEO/MP4"),
            Some(MediaKind::Video)
        );
        assert_eq!(media_kind_from_content_type("application/pdf"), None);
    }
}
