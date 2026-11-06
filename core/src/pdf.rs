//! Mini-générateur PDF sans dépendance (polices standard Helvetica/Courier, WinAnsi).
//! Suffisant pour un rapport tabulaire ; les vignettes viendront en V3.

use std::fmt::Write as _;

pub const W: f32 = 595.0; // A4 en points
pub const H: f32 = 842.0;

#[derive(Clone, Copy)]
pub enum Font {
    Regular,
    Bold,
    Mono,
}
impl Font {
    fn res(self) -> &'static str {
        match self {
            Font::Regular => "F1",
            Font::Bold => "F2",
            Font::Mono => "F3",
        }
    }
}

#[derive(Default)]
pub struct Pdf {
    pages: Vec<String>,
}

impl Pdf {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn new_page(&mut self) {
        self.pages.push(String::new());
    }
    fn cur(&mut self) -> &mut String {
        if self.pages.is_empty() {
            self.new_page();
        }
        self.pages.last_mut().unwrap()
    }
    /// Texte avec origine en haut à gauche (y descend).
    pub fn text(&mut self, x: f32, y: f32, size: f32, font: Font, rgb: (f32, f32, f32), s: &str) {
        let enc = encode(s);
        let c = self.cur();
        let _ = writeln!(
            c,
            "BT /{} {size} Tf {} {} {} rg {x:.2} {:.2} Td ({enc}) Tj ET",
            font.res(),
            rgb.0,
            rgb.1,
            rgb.2,
            H - y
        );
    }
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, rgb: (f32, f32, f32)) {
        let c = self.cur();
        let _ = writeln!(c, "{} {} {} rg {x:.2} {:.2} {w:.2} {h:.2} re f", rgb.0, rgb.1, rgb.2, H - y - h);
    }
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, rgb: (f32, f32, f32), width: f32) {
        let c = self.cur();
        let _ = writeln!(
            c,
            "{} {} {} RG {width} w {x1:.2} {:.2} m {x2:.2} {:.2} l S",
            rgb.0,
            rgb.1,
            rgb.2,
            H - y1,
            H - y2
        );
    }

    pub fn render(&self) -> Vec<u8> {
        let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets: Vec<usize> = vec![];
        let n_pages = self.pages.len().max(1);
        // 1 catalog, 2 pages, 3-5 fonts, puis (page, contenu) par page
        let mut objs: Vec<Vec<u8>> = vec![];
        objs.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
        let kids: Vec<String> = (0..n_pages).map(|i| format!("{} 0 R", 6 + i * 2)).collect();
        objs.push(format!("<< /Type /Pages /Kids [{}] /Count {n_pages} >>", kids.join(" ")).into_bytes());
        for base in ["Helvetica", "Helvetica-Bold", "Courier"] {
            objs.push(
                format!("<< /Type /Font /Subtype /Type1 /BaseFont /{base} /Encoding /WinAnsiEncoding >>")
                    .into_bytes(),
            );
        }
        let empty = String::new();
        for i in 0..n_pages {
            let content = self.pages.get(i).unwrap_or(&empty);
            objs.push(
                format!(
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {W} {H}] \
                     /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R >> >> /Contents {} 0 R >>",
                    7 + i * 2
                )
                .into_bytes(),
            );
            let mut s = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
            s.extend_from_slice(content.as_bytes());
            s.extend_from_slice(b"\nendstream");
            objs.push(s);
        }
        for (i, o) in objs.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend_from_slice(o);
            out.extend_from_slice(b"\nendobj\n");
        }
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
        for off in offsets {
            out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes(),
        );
        out
    }
}

/// Encode en WinAnsi (Latin-1 + quelques signes) avec échappement PDF.
/// Le contenu reste ASCII dans le flux grâce aux échappements octaux.
fn encode(s: &str) -> String {
    let mut o = String::new();
    for ch in s.chars() {
        let b: u32 = match ch {
            '€' => 0x80,
            '’' => 0x92,
            '“' => 0x93,
            '”' => 0x94,
            '•' => 0x95,
            '–' => 0x96,
            '…' => 0x85,
            c if (c as u32) < 0x100 => c as u32,
            _ => '?' as u32,
        };
        match b {
            0x28 => o.push_str("\\("),
            0x29 => o.push_str("\\)"),
            0x5c => o.push_str("\\\\"),
            0x20..=0x7e => o.push(b as u8 as char),
            _ => {
                let _ = write!(o, "\\{b:03o}");
            }
        }
    }
    o
}

/// Largeur approximative (Helvetica ≈ 0.5 em en moyenne, Courier = 0.6 em).
pub fn approx_width(s: &str, size: f32, font: Font) -> f32 {
    let k = match font {
        Font::Mono => 0.6,
        Font::Bold => 0.56,
        Font::Regular => 0.52,
    };
    s.chars().count() as f32 * size * k
}

/// Tronque au milieu pour tenir dans une largeur donnée.
pub fn fit(s: &str, max_w: f32, size: f32, font: Font) -> String {
    if approx_width(s, size, font) <= max_w {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let keep = ((max_w / (size * 0.55)) as usize).saturating_sub(1).max(4);
    let head = keep * 2 / 3;
    let tail = keep - head;
    let mut r: String = chars[..head.min(chars.len())].iter().collect();
    r.push('…');
    r.extend(&chars[chars.len().saturating_sub(tail)..]);
    r
}
