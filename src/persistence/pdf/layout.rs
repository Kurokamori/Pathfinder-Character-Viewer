//! Page layout primitives for the PDF character-sheet exporter.
//!
//! Wraps `printpdf` in a top-down cursor model: callers emit headings, rows and
//! paragraphs in order and the writer handles pagination, footers and text
//! wrapping. Coordinates are millimetres measured from the bottom-left of the
//! page, matching printpdf's own convention.

use printpdf::{
    BuiltinFont, Color, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerReference, Point, Rgb,
};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub const PAGE_W: f32 = 210.0;
pub const PAGE_H: f32 = 297.0;
pub const MARGIN_X: f32 = 15.0;
pub const CONTENT_W: f32 = PAGE_W - 2.0 * MARGIN_X;

const TOP_Y: f32 = 281.0;
const BOTTOM_Y: f32 = 16.0;
const FOOTER_Y: f32 = 10.0;
const PT_TO_MM: f32 = 0.352_777_8;

/// Which builtin face a run of text uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Regular,
    Bold,
    Italic,
}

/// Greyscale ink levels used across the sheet.
const INK: f32 = 0.10;
const MUTED: f32 = 0.42;
const RULE: f32 = 0.72;

fn grey(level: f32) -> Color {
    Color::Rgb(Rgb::new(level, level, level, None))
}

/// Estimated rendered width of `text` in millimetres.
///
/// The builtin PDF fonts expose no metrics table through printpdf, so this
/// approximates Helvetica's average advance width. It is used only to decide
/// line breaks, where a small overestimate is harmless.
pub fn text_width(text: &str, size: f32, style: Style) -> f32 {
    let factor: f32 = match style {
        Style::Bold => 0.55,
        _ => 0.50,
    };
    text.chars().count() as f32 * size * factor * PT_TO_MM
}

/// Fold text into the WinAnsi range the builtin fonts can encode.
///
/// Compendium prose carries typographic punctuation that would otherwise be
/// dropped or mangled by the builtin font encoding, so map the common cases to
/// ASCII equivalents and replace anything else outside Latin-1.
pub fn encode(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
            '\u{2026}' => ' ',
            '\u{00A0}' | '\u{2007}' | '\u{202F}' => ' ',
            '\u{2022}' | '\u{00B7}' => '-',
            '\t' => ' ',
            c if (c as u32) < 32 => ' ',
            c if (c as u32) > 255 => '?',
            c => c,
        })
        .collect()
}

/// A cell in a fixed-column row: an x offset from the left margin plus content.
pub struct Cell {
    pub x: f32,
    pub text: String,
    pub style: Style,
}

impl Cell {
    pub fn new(x: f32, text: impl Into<String>) -> Self {
        Cell {
            x,
            text: text.into(),
            style: Style::Regular,
        }
    }

    pub fn bold(x: f32, text: impl Into<String>) -> Self {
        Cell {
            x,
            text: text.into(),
            style: Style::Bold,
        }
    }
}

/// A paginating, top-down PDF writer.
pub struct Pdf {
    doc: PdfDocumentReference,
    layer: PdfLayerReference,
    regular: IndirectFontRef,
    bold: IndirectFontRef,
    italic: IndirectFontRef,
    footer: String,
    page_no: u32,
    y: f32,
}

impl Pdf {
    pub fn new(title: &str, footer: &str) -> Result<Self, printpdf::Error> {
        let (doc, page, layer) = PdfDocument::new(encode(title), Mm(PAGE_W), Mm(PAGE_H), "Sheet");
        let regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let italic = doc.add_builtin_font(BuiltinFont::HelveticaOblique)?;
        let layer = doc.get_page(page).get_layer(layer);

        let pdf = Pdf {
            doc,
            layer,
            regular,
            bold,
            italic,
            footer: encode(footer),
            page_no: 1,
            y: TOP_Y,
        };
        pdf.draw_footer();
        Ok(pdf)
    }

    fn font(&self, style: Style) -> &IndirectFontRef {
        match style {
            Style::Regular => &self.regular,
            Style::Bold => &self.bold,
            Style::Italic => &self.italic,
        }
    }

    fn draw_footer(&self) {
        self.layer.set_fill_color(grey(MUTED));
        self.layer.use_text(
            self.footer.clone(),
            8.0,
            Mm(MARGIN_X),
            Mm(FOOTER_Y),
            &self.regular,
        );
        let label = format!("Page {}", self.page_no);
        let x = PAGE_W - MARGIN_X - text_width(&label, 8.0, Style::Regular);
        self.layer
            .use_text(label, 8.0, Mm(x), Mm(FOOTER_Y), &self.regular);
        self.layer.set_fill_color(grey(INK));
    }

    pub fn new_page(&mut self) {
        let (page, layer) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Sheet");
        self.layer = self.doc.get_page(page).get_layer(layer);
        self.page_no += 1;
        self.y = TOP_Y;
        self.draw_footer();
    }

    /// Break to a new page unless `needed` millimetres remain.
    pub fn ensure(&mut self, needed: f32) {
        if self.y - needed < BOTTOM_Y {
            self.new_page();
        }
    }

    pub fn advance(&mut self, mm: f32) {
        self.y -= mm;
    }

    fn draw_text(&self, text: &str, size: f32, x: f32, y: f32, style: Style) {
        if text.is_empty() {
            return;
        }
        self.layer
            .use_text(encode(text), size, Mm(x), Mm(y), self.font(style));
    }

    fn rule(&self, y: f32, level: f32, thickness: f32) {
        self.layer.set_outline_color(grey(level));
        self.layer.set_outline_thickness(thickness);
        self.layer.add_line(Line {
            points: vec![
                (Point::new(Mm(MARGIN_X), Mm(y)), false),
                (Point::new(Mm(PAGE_W - MARGIN_X), Mm(y)), false),
            ],
            is_closed: false,
        });
    }

    /// The character name and subtitle at the very top of page one.
    pub fn title(&mut self, name: &str, subtitle: &str) {
        self.draw_text(name, 22.0, MARGIN_X, self.y, Style::Bold);
        self.advance(7.5);
        if !subtitle.is_empty() {
            self.layer.set_fill_color(grey(MUTED));
            self.draw_text(subtitle, 10.5, MARGIN_X, self.y, Style::Regular);
            self.layer.set_fill_color(grey(INK));
            self.advance(4.5);
        }
        self.rule(self.y, INK, 0.8);
        self.advance(7.0);
    }

    /// A major section heading with an underline, kept with the block after it.
    pub fn section(&mut self, label: &str) {
        self.ensure(24.0);
        self.advance(2.0);
        self.draw_text(&label.to_uppercase(), 11.5, MARGIN_X, self.y, Style::Bold);
        self.advance(2.2);
        self.rule(self.y, RULE, 0.4);
        self.advance(5.5);
    }

    pub fn subheading(&mut self, label: &str) {
        self.ensure(12.0);
        self.draw_text(label, 9.5, MARGIN_X, self.y, Style::Bold);
        self.advance(5.0);
    }

    /// A muted, italic explanatory line.
    pub fn caption(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.ensure(8.0);
        self.layer.set_fill_color(grey(MUTED));
        for line in wrap(text, CONTENT_W, 8.5, Style::Italic) {
            self.ensure(5.0);
            self.draw_text(&line, 8.5, MARGIN_X, self.y, Style::Italic);
            self.advance(4.2);
        }
        self.layer.set_fill_color(grey(INK));
        self.advance(1.0);
    }

    pub fn row(&mut self, cells: &[Cell], size: f32) {
        self.ensure(7.0);
        for cell in cells {
            self.draw_text(&cell.text, size, MARGIN_X + cell.x, self.y, cell.style);
        }
        self.advance(size * PT_TO_MM + 2.4);
    }

    /// A row rendered in muted small caps, used for table column headers.
    pub fn header_row(&mut self, cells: &[Cell]) {
        self.ensure(9.0);
        self.layer.set_fill_color(grey(MUTED));
        for cell in cells {
            self.draw_text(
                &cell.text.to_uppercase(),
                7.5,
                MARGIN_X + cell.x,
                self.y,
                Style::Bold,
            );
        }
        self.layer.set_fill_color(grey(INK));
        self.advance(3.2);
        self.rule(self.y, RULE, 0.25);
        self.advance(4.0);
    }

    /// Label/value pairs laid out across `cols` equal columns.
    pub fn grid(&mut self, pairs: &[(String, String)], cols: usize) {
        if pairs.is_empty() {
            return;
        }
        let cols = cols.max(1);
        let col_w = CONTENT_W / cols as f32;
        for chunk in pairs.chunks(cols) {
            self.ensure(8.0);
            for (i, (label, value)) in chunk.iter().enumerate() {
                let x = MARGIN_X + col_w * i as f32;
                self.layer.set_fill_color(grey(MUTED));
                self.draw_text(label, 8.0, x, self.y, Style::Regular);
                self.layer.set_fill_color(grey(INK));
                let label_w = text_width(label, 8.0, Style::Regular);
                self.draw_text(value, 9.5, x + label_w + 2.0, self.y, Style::Bold);
            }
            self.advance(6.0);
        }
        self.advance(1.0);
    }

    /// A wrapped prose block, optionally indented.
    pub fn paragraph(&mut self, text: &str, size: f32, indent: f32) {
        let width = CONTENT_W - indent;
        for raw in text.split('\n') {
            let trimmed = raw.trim_end();
            if trimmed.trim().is_empty() {
                self.advance(2.5);
                continue;
            }
            for line in wrap(trimmed, width, size, Style::Regular) {
                self.ensure(6.0);
                self.draw_text(&line, size, MARGIN_X + indent, self.y, Style::Regular);
                self.advance(size * PT_TO_MM + 1.9);
            }
        }
        self.advance(1.5);
    }

    /// A named block of prose: bold label, then the body indented beneath it.
    pub fn labelled_block(&mut self, label: &str, body: &str) {
        if body.trim().is_empty() {
            return;
        }
        self.ensure(16.0);
        self.subheading(label);
        self.paragraph(body, 9.0, 3.0);
        self.advance(1.5);
    }

    pub fn bullet(&mut self, text: &str) {
        self.ensure(7.0);
        self.draw_text("-", 9.0, MARGIN_X, self.y, Style::Regular);
        let width = CONTENT_W - 4.0;
        let lines = wrap(text, width, 9.0, Style::Regular);
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                self.ensure(6.0);
            }
            self.draw_text(line, 9.0, MARGIN_X + 4.0, self.y, Style::Regular);
            self.advance(9.0 * PT_TO_MM + 1.9);
        }
    }

    pub fn save(self, path: &Path) -> Result<(), printpdf::Error> {
        let file = File::create(path)?;
        self.doc.save(&mut BufWriter::new(file))
    }
}

/// Greedily break `text` into lines no wider than `width` millimetres.
///
/// Words longer than the line width are hard-split so a single long token can
/// never push content past the margin.
pub fn wrap(text: &str, width: f32, size: f32, style: Style) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if text_width(&candidate, size, style) <= width || current.is_empty() {
            current = candidate;
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }

        while text_width(&current, size, style) > width && current.chars().count() > 1 {
            let keep = fitting_prefix(&current, width, size, style);
            let split = current.chars().take(keep).collect::<String>();
            current = current.chars().skip(keep).collect();
            lines.push(split);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// How many leading characters of `text` fit within `width`.
fn fitting_prefix(text: &str, width: f32, size: f32, style: Style) -> usize {
    let total = text.chars().count();
    let mut keep = total;
    while keep > 1 {
        let prefix: String = text.chars().take(keep).collect();
        if text_width(&prefix, size, style) <= width {
            break;
        }
        keep -= 1;
    }
    keep
}
