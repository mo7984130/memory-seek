use std::{
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
};

use bytes::Bytes;
use image::{ImageError, ImageFormat};
use nom_exif::{MediaParser, MediaSource, TrackInfoTag};

pub const IMAGE_MAX_SIZE: u64 = 32 * 1024 * 1024;
pub const VIDEO_MAX_SIZE: u64 = 1024 * 1024 * 1024;
/// 嗅探类型时需要读取的头部字节数(需覆盖 WebM/MKV 的 DocType)
pub const SNIFF_HEAD_LEN: usize = 128;

#[derive(Debug, thiserror::Error)]
pub enum MediaValidatorError {
    #[error("文件不可为空")]
    EmptyFile,
    // 单位应该为MB
    #[error("文件过大, 不可超过 {} MB", 0)]
    TooLarge(u64),
    #[error("不支持的文件类型")]
    UnsupportedType,
    #[error("解析图片失败: {}", 0)]
    ImageParseError(#[from] ImageError),
    #[error("解析视频失败: {}", 0)]
    VideoParseError(#[from] nom_exif::Error),
    #[error("{}", 0)]
    InvalidMedia(&'static str),
    #[error("读取失败: {}", 0)]
    ReadError(#[from] io::Error),
}

type Error = MediaValidatorError;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageMediaKind {
    format: ImageFormat,
    ext: &'static str,
    mime_type: &'static str,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoMediaKind {
    ext: &'static str,
    mime_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image(ImageMediaKind),
    Video(VideoMediaKind),
}

impl MediaKind {
    pub const JPEG: Self = Self::Image(ImageMediaKind {
        ext: "jpg",
        mime_type: "image/jpeg",
        format: ImageFormat::Jpeg,
    });
    pub const PNG: Self = Self::Image(ImageMediaKind {
        ext: "png",
        mime_type: "image/png",
        format: ImageFormat::Png,
    });
    pub const WEBP: Self = Self::Image(ImageMediaKind {
        ext: "webp",
        mime_type: "image/webp",
        format: ImageFormat::WebP,
    });

    pub const MP4: Self = Self::Video(VideoMediaKind {
        ext: "mp4",
        mime_type: "video/mp4",
    });
    pub const MOV: Self = Self::Video(VideoMediaKind {
        ext: "mov",
        mime_type: "video/quicktime",
    });
    pub const WEBM: Self = Self::Video(VideoMediaKind {
        ext: "webm",
        mime_type: "video/webm",
    });
    pub const MKV: Self = Self::Video(VideoMediaKind {
        ext: "mkv",
        mime_type: "video/x-matroska",
    });

    pub fn ext(&self) -> &'static str {
        match self {
            Self::Image(k) => k.ext,
            Self::Video(k) => k.ext,
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Image(k) => k.mime_type,
            Self::Video(k) => k.mime_type,
        }
    }
}

/// ISOBMFF(MP4/MOV) 魔数
const ISOBMFF_TAG: [u8; 4] = [102, 116, 121, 112];
const QT_BRAND: [u8; 4] = [113, 116, 32, 32];
/// EBML(WebM/MKV) 魔数
const EBML_MAGIC: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];

/// 图片与视频统一解析后的元数据
#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub width: u32,
    pub height: u32,
    /// 视频时长(毫秒);图片为 `None`
    pub duration_ms: Option<u64>,
    pub size: u64,
    pub kind: MediaKind,
}

pub struct MediaValidator;
impl MediaValidator {
    pub fn sniff(head: &[u8]) -> Option<MediaKind> {
        Self::sniff_image_format(head).or_else(|| Self::sniff_video_format(head))
    }

    pub fn sniff_ensure(head: &[u8]) -> Result<MediaKind> {
        Self::sniff(head).ok_or(Error::UnsupportedType)
    }

    pub fn sniff_image_format(head: &[u8]) -> Option<MediaKind> {
        match image::guess_format(head).ok()? {
            image::ImageFormat::Jpeg => Some(MediaKind::JPEG),
            image::ImageFormat::Png => Some(MediaKind::PNG),
            image::ImageFormat::WebP => Some(MediaKind::WEBP),
            _ => None,
        }
    }

    pub fn sniff_video_format(head: &[u8]) -> Option<MediaKind> {
        if head.len() >= 12 && head[4..8] == ISOBMFF_TAG {
            return if head[8..12] == QT_BRAND {
                Some(MediaKind::MOV)
            } else {
                Some(MediaKind::MP4)
            };
        }
        if head.len() >= EBML_MAGIC.len() && head[..EBML_MAGIC.len()] == EBML_MAGIC {
            if contains_ascii(head, b"webm") {
                return Some(MediaKind::WEBM);
            }
            if contains_ascii(head, b"matroska") {
                return Some(MediaKind::MKV);
            }
            return None;
        }
        None
    }

    pub fn validate(bytes: Bytes) -> Result<MediaMetadata> {
        if bytes.is_empty() {
            return Err(Error::EmptyFile);
        }
        let size = bytes.len() as u64;
        match Self::sniff_ensure(&bytes)? {
            MediaKind::Image(image_kind) => {
                Self::do_validate_image(Cursor::new(bytes.as_ref()), image_kind, size)
            }
            MediaKind::Video(video_kind) => {
                Self::do_validate_video(video_kind, MediaSource::from_memory(bytes)?, size)
            }
        }
    }

    pub fn validate_image(bytes: Bytes) -> Result<MediaMetadata> {
        if bytes.is_empty() {
            return Err(Error::EmptyFile);
        }
        let size = bytes.len() as u64;
        match Self::sniff_ensure(&bytes)? {
            MediaKind::Image(image_kind) => {
                Self::do_validate_image(Cursor::new(bytes.as_ref()), image_kind, size)
            }
            MediaKind::Video(_) => Err(Error::UnsupportedType),
        }
    }

    pub fn validate_file(file_path: &Path) -> Result<MediaMetadata> {
        let file = File::open(file_path)?;
        let size = file.metadata()?.len();
        if size == 0 {
            return Err(Error::EmptyFile);
        }

        let mut reader = BufReader::new(file);
        let mut head = [0u8; SNIFF_HEAD_LEN];
        let n = read_full(&mut reader, &mut head)?;
        let kind = Self::sniff_ensure(&head[..n])?;

        reader.seek(SeekFrom::Start(0))?;

        match kind {
            MediaKind::Image(image_kind) => Self::do_validate_image(reader, image_kind, size),
            MediaKind::Video(video_kind) => {
                Self::do_validate_video(video_kind, MediaSource::seekable(reader)?, size)
            }
        }
    }

    fn do_validate_image<R: BufRead + Seek>(
        reader: R,
        kind: ImageMediaKind,
        size: u64,
    ) -> Result<MediaMetadata> {
        if size > IMAGE_MAX_SIZE {
            return Err(Error::TooLarge(IMAGE_MAX_SIZE / 1024 / 1024));
        }

        let (width, height) =
            image::ImageReader::with_format(reader, kind.format).into_dimensions()?;

        Ok(MediaMetadata {
            width,
            height,
            duration_ms: None,
            size,
            kind: MediaKind::Image(kind),
        })
    }

    fn do_validate_video<R: Read>(
        kind: VideoMediaKind,
        media_source: MediaSource<R>,
        size: u64,
    ) -> Result<MediaMetadata> {
        if size > VIDEO_MAX_SIZE {
            return Err(Error::TooLarge(VIDEO_MAX_SIZE / 1024 / 1024));
        }

        let track_info = MediaParser::new().parse_track(media_source)?;

        let width = track_info
            .get(TrackInfoTag::Width)
            .and_then(|value| value.as_u32())
            .filter(|&width| width > 0)
            .ok_or_else(|| MediaValidatorError::InvalidMedia("无法解析视频宽度"))?;
        let height = track_info
            .get(TrackInfoTag::Height)
            .and_then(|value| value.as_u32())
            .filter(|&height| height > 0)
            .ok_or_else(|| MediaValidatorError::InvalidMedia("无法解析视频高度"))?;
        let duration_ms = track_info
            .get(TrackInfoTag::DurationMs)
            .and_then(|value| value.as_u64())
            .filter(|&duration_ms| duration_ms > 0)
            .ok_or_else(|| MediaValidatorError::InvalidMedia("无法解析视频时长"))?;

        Ok(MediaMetadata {
            width,
            height,
            duration_ms: Some(duration_ms),
            size,
            kind: MediaKind::Video(kind),
        })
    }
}

/// 在切片中查找 ASCII 子串
fn contains_ascii(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// 读取填充 `buf`, 返回实际读取的字节数(文件不足 `buf` 长度时)
fn read_full<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<usize> {
    let mut read = 0;
    while read < buf.len() {
        match reader.read(&mut buf[read..])? {
            0 => break,
            n_read => read += n_read,
        }
    }
    Ok(read)
}
