use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Text,
    Image,
    Link,
    Color,
    Code,
}

pub fn classify(plain: Option<&str>, has_image: bool) -> Kind {
    let s = plain.unwrap_or("").trim();
    if s.is_empty() && has_image {
        return Kind::Image;
    }
    if is_link(s) {
        return Kind::Link;
    }
    if is_color(s) {
        return Kind::Color;
    }
    if is_code(s) {
        return Kind::Code;
    }
    Kind::Text
}

const CODE_KEYWORDS: &[&str] = &["SELECT", "INSERT", "UPDATE", "DELETE", "WITH", "CREATE", "ALTER", "using", "namespace", "public", "private", "class", "function", "const", "let", "import", "export", "fn", "def"];

fn is_code(s: &str) -> bool {
    let first_word = s.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("").split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
    if CODE_KEYWORDS.contains(&first_word) {
        return true;
    }
    let visible = s.chars().filter(|c| !c.is_whitespace()).count();
    let symbols = s.chars().filter(|c| "{}();".contains(*c)).count();
    s.lines().count() >= 2 && symbols * 20 >= visible
}

fn is_color(s: &str) -> bool {
    if let Some(hex) = s.strip_prefix('#') {
        return matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    let lower = s.to_ascii_lowercase();
    ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|p| {
        lower.strip_prefix(p).and_then(|r| r.strip_suffix(')')).is_some_and(|args| {
            args.chars().any(|c| c.is_ascii_digit()) && args.chars().all(|c| c.is_ascii_digit() || " ,.%/".contains(c))
        })
    })
}

fn is_link(s: &str) -> bool {
    ["http://", "https://", "ftp://", "mailto:", "www."].iter().any(|p| s.starts_with(p)) && !s.contains(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links() {
        assert_eq!(classify(Some("  https://example.com/a?b=1 \n"), false), Kind::Link);
        assert_eq!(classify(Some("www.example.com"), false), Kind::Link);
        assert_eq!(classify(Some("mailto:a@b.co"), false), Kind::Link);
        assert_eq!(classify(Some("https://a.com https://b.com"), false), Kind::Text);
    }

    #[test]
    fn colors() {
        for s in ["#fff", "#1B1B1E", "#1b1b1eff", "rgb(1, 2, 3)", "rgba(1,2,3,0.5)", "hsl(120, 50%, 50%)", "hsla(120,50%,50%,.3)"] {
            assert_eq!(classify(Some(s), false), Kind::Color, "{s}");
        }
        assert_eq!(classify(Some("#12345"), false), Kind::Text);
    }

    #[test]
    fn image_only_without_text() {
        assert_eq!(classify(None, true), Kind::Image);
        assert_eq!(classify(Some("A1\tB1"), true), Kind::Text);
    }

    #[test]
    fn code() {
        assert_eq!(classify(Some("SELECT * FROM \"Orders\"\nWHERE \"Id\" = 1;"), false), Kind::Code);
        assert_eq!(classify(Some("fn main() {\n    println!(\"x\");\n}"), false), Kind::Code);
        assert_eq!(classify(Some("public class A\n{\n    int x;\n}"), false), Kind::Code);
        assert_eq!(classify(Some("Merhaba, yarın görüşürüz."), false), Kind::Text);
        assert_eq!(classify(Some("Toplantı notları:\n- madde bir\n- madde iki"), false), Kind::Text);
    }
}
