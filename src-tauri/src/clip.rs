use crate::keys::PasteMode;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ClipFormats {
    pub plain: Option<String>,
    pub html: Option<Vec<u8>>,
    pub rtf: Option<Vec<u8>>,
    pub png: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq)]
pub enum OutFormat {
    Plain(String),
    Html(Vec<u8>),
    Rtf(Vec<u8>),
    Image(Vec<u8>),
    ExcludeMonitor,
    CanIncludeInHistory(u32),
}

pub fn write_plan(f: &ClipFormats, mode: PasteMode, private: bool) -> Vec<OutFormat> {
    let mut out: Vec<OutFormat> = f.plain.clone().map(OutFormat::Plain).into_iter().collect();
    if mode != PasteMode::PlainPaste || f.plain.is_none() {
        out.extend(f.html.clone().map(OutFormat::Html));
        out.extend(f.rtf.clone().map(OutFormat::Rtf));
        out.extend(f.png.clone().map(OutFormat::Image));
    }
    if private {
        out.extend([OutFormat::ExcludeMonitor, OutFormat::CanIncludeInHistory(0)]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn full() -> ClipFormats { ClipFormats { plain: Some("a".into()), html: Some(b"<b>a</b>".to_vec()), rtf: Some(b"{\\rtf1 a}".to_vec()), png: Some(vec![1, 2]) } }

    #[test]
    fn paste_writes_all_formats() {
        let p = write_plan(&full(), PasteMode::Paste, false);
        assert_eq!(p, vec![OutFormat::Plain("a".into()), OutFormat::Html(b"<b>a</b>".to_vec()), OutFormat::Rtf(b"{\\rtf1 a}".to_vec()), OutFormat::Image(vec![1, 2])]);
        assert_eq!(write_plan(&full(), PasteMode::Copy, false), p);
    }

    #[test]
    fn plain_paste_writes_only_text() {
        assert_eq!(write_plan(&full(), PasteMode::PlainPaste, false), vec![OutFormat::Plain("a".into())]);
    }

    #[test]
    fn private_adds_history_exclusion() {
        let p = write_plan(&full(), PasteMode::PlainPaste, true);
        assert!(p.contains(&OutFormat::ExcludeMonitor));
        assert!(p.contains(&OutFormat::CanIncludeInHistory(0)));
    }

    #[test]
    fn plain_paste_of_image_only_item_keeps_image() {
        let f = ClipFormats { png: Some(vec![1]), ..Default::default() };
        assert_eq!(write_plan(&f, PasteMode::PlainPaste, false), vec![OutFormat::Image(vec![1])]);
    }
}
