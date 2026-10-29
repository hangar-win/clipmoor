use crate::clip::ClipFormats;

pub const TEXT_MAX: usize = 5 * 1024 * 1024;
pub const IMAGE_MAX: usize = 25 * 1024 * 1024;
pub const RICH_MAX: usize = 5 * 1024 * 1024;
pub const THUMB_HEIGHT: u32 = 96;
const IMAGE_SIDE_MAX: u32 = 16_384;
const IMAGE_ALLOC_MAX: u64 = 256 * 1024 * 1024;

pub fn image_limits() -> image::Limits {
    let mut l = image::Limits::default();
    l.max_image_width = Some(IMAGE_SIDE_MAX);
    l.max_image_height = Some(IMAGE_SIDE_MAX);
    l.max_alloc = Some(IMAGE_ALLOC_MAX);
    l
}

pub fn decode_png(png: &[u8]) -> Option<image::DynamicImage> {
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(png), image::ImageFormat::Png);
    reader.limits(image_limits());
    reader.decode().ok()
}

#[derive(Default, Debug)]
pub struct ClipMarkers {
    pub exclude_monitor: bool,
    pub viewer_ignore: bool,
    pub can_include_in_history: Option<u32>,
}

#[derive(Debug)]
pub struct RawClip {
    pub formats: ClipFormats,
    pub markers: ClipMarkers,
    pub seq: u32,
    pub source_exe: String,
}

pub struct CaptureContext<'a> {
    pub excluded_apps: &'a [String],
    pub paused: bool,
    pub own_seq: Option<u32>,
}

#[derive(Debug, PartialEq)]
pub enum SkipReason {
    Marked,
    Excluded,
    Paused,
    Own,
    TooLarge,
    Empty,
}

#[derive(Debug, PartialEq)]
pub enum CaptureDecision {
    Skip(SkipReason),
    Store(ClipFormats),
}

pub fn decide_capture(raw: RawClip, ctx: &CaptureContext) -> CaptureDecision {
    let m = &raw.markers;
    if m.exclude_monitor || m.viewer_ignore || m.can_include_in_history == Some(0) {
        return CaptureDecision::Skip(SkipReason::Marked);
    }
    if ctx.own_seq == Some(raw.seq) {
        return CaptureDecision::Skip(SkipReason::Own);
    }
    if ctx.paused {
        return CaptureDecision::Skip(SkipReason::Paused);
    }
    if ctx.excluded_apps.iter().any(|a| a.eq_ignore_ascii_case(&raw.source_exe)) {
        return CaptureDecision::Skip(SkipReason::Excluded);
    }
    let mut f = raw.formats;
    f.plain = f.plain.filter(|p| !p.is_empty());
    if f.plain.is_none() && f.png.is_none() {
        return CaptureDecision::Skip(SkipReason::Empty);
    }
    if f.plain.as_ref().is_some_and(|p| p.len() > TEXT_MAX) {
        return CaptureDecision::Skip(SkipReason::TooLarge);
    }
    if f.png.as_ref().is_some_and(|p| p.len() > IMAGE_MAX) {
        f.png = None;
    }
    if f.plain.is_none() && f.png.is_none() {
        return CaptureDecision::Skip(SkipReason::TooLarge);
    }
    if f.html.as_ref().map_or(0, Vec::len) + f.rtf.as_ref().map_or(0, Vec::len) > RICH_MAX {
        f.html = None;
        f.rtf = None;
    }
    CaptureDecision::Store(f)
}

pub fn thumbnail(png: &[u8]) -> Option<Vec<u8>> {
    let img = decode_png(png)?;
    if img.height() <= THUMB_HEIGHT {
        return None;
    }
    let width = (u64::from(img.width()) * u64::from(THUMB_HEIGHT) / u64::from(img.height())).max(1) as u32;
    let mut out = std::io::Cursor::new(Vec::new());
    image::imageops::thumbnail(&img, width, THUMB_HEIGHT).write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(plain: Option<&str>) -> RawClip {
        RawClip { formats: ClipFormats { plain: plain.map(Into::into), ..Default::default() }, markers: ClipMarkers::default(), seq: 10, source_exe: "notepad.exe".into() }
    }
    fn ctx() -> CaptureContext<'static> { CaptureContext { excluded_apps: &[], paused: false, own_seq: None } }

    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        image::RgbaImage::new(w, h).write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn skips_password_manager_markers() {
        let mut r = raw(Some("p"));
        r.markers.exclude_monitor = true;
        assert_eq!(decide_capture(r, &ctx()), CaptureDecision::Skip(SkipReason::Marked));
        let mut r = raw(Some("p"));
        r.markers.can_include_in_history = Some(0);
        assert_eq!(decide_capture(r, &ctx()), CaptureDecision::Skip(SkipReason::Marked));
        let mut r = raw(Some("p"));
        r.markers.viewer_ignore = true;
        assert_eq!(decide_capture(r, &ctx()), CaptureDecision::Skip(SkipReason::Marked));
        let mut r = raw(Some("p"));
        r.markers.can_include_in_history = Some(1);
        assert!(matches!(decide_capture(r, &ctx()), CaptureDecision::Store(_)));
    }

    #[test]
    fn skips_excluded_paused_and_own() {
        let ex = vec!["KeePassXC.exe".to_string()];
        let mut r = raw(Some("p"));
        r.source_exe = "keepassxc.exe".into();
        assert_eq!(decide_capture(r, &CaptureContext { excluded_apps: &ex, paused: false, own_seq: None }), CaptureDecision::Skip(SkipReason::Excluded));
        assert_eq!(decide_capture(raw(Some("p")), &CaptureContext { excluded_apps: &[], paused: true, own_seq: None }), CaptureDecision::Skip(SkipReason::Paused));
        assert_eq!(decide_capture(raw(Some("p")), &CaptureContext { excluded_apps: &[], paused: false, own_seq: Some(10) }), CaptureDecision::Skip(SkipReason::Own));
    }

    #[test]
    fn oversize_rich_keeps_plain() {
        let mut r = raw(Some("p"));
        r.formats.html = Some(vec![b'a'; RICH_MAX + 1]);
        r.formats.rtf = Some(b"{\\rtf1}".to_vec());
        let CaptureDecision::Store(f) = decide_capture(r, &ctx()) else { panic!() };
        assert_eq!((f.plain.as_deref(), f.html.is_none(), f.rtf.is_none()), (Some("p"), true, true));
    }

    #[test]
    fn oversize_text_or_image_dropped() {
        assert_eq!(decide_capture(raw(Some("a".repeat(TEXT_MAX + 1).as_str())), &ctx()), CaptureDecision::Skip(SkipReason::TooLarge));
        let mut r = raw(None);
        r.formats.png = Some(vec![0; IMAGE_MAX + 1]);
        assert_eq!(decide_capture(r, &ctx()), CaptureDecision::Skip(SkipReason::TooLarge));
    }

    #[test]
    fn oversize_image_with_text_keeps_text() {
        let mut r = raw(Some("A1"));
        r.formats.png = Some(vec![0; IMAGE_MAX + 1]);
        let CaptureDecision::Store(f) = decide_capture(r, &ctx()) else { panic!() };
        assert_eq!((f.plain.as_deref(), f.png.is_none()), (Some("A1"), true));
    }

    #[test]
    fn empty_clip_skipped() {
        assert_eq!(decide_capture(raw(None), &ctx()), CaptureDecision::Skip(SkipReason::Empty));
        assert_eq!(decide_capture(raw(Some("")), &ctx()), CaptureDecision::Skip(SkipReason::Empty));
    }

    #[test]
    fn thumbnail_is_96_high_and_keeps_aspect() {
        let t = image::load_from_memory(&thumbnail(&png(400, 200)).unwrap()).unwrap();
        assert_eq!((t.width(), t.height()), (192, 96));
        assert!(thumbnail(&png(40, 20)).is_none());
        assert!(thumbnail(b"not a png").is_none());
    }
}
