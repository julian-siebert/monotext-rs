use printpdf::{
    BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Pt, TextItem,
    TextMatrix,
};

use crate::{Config, Document};

pub fn write_pdf(cfg: Config, document: Document) -> anyhow::Result<Vec<u8>> {
    let mut pdf = PdfDocument::new(&document.title);

    let rendered = document.render(cfg);
    let pages: Vec<&str> = rendered.split('\x0C').collect();

    let font = BuiltinFont::Courier;

    // A4 size (mm)
    let page_w = Mm(210.0);
    let page_h = Mm(297.0);

    let font_size = 10.0;

    let char_w = font_size * 0.6 * 0.3528; // pt → mm
    let line_h = font_size * 1.2 * 0.3528;

    let block_w = char_w * cfg.page_width as f32;
    let block_h = line_h * cfg.page_height as f32;

    let margin_x = (page_w.0 - block_w) / 2.0;
    let margin_y = (page_h.0 - block_h) / 2.0;

    for page_text in pages {
        let mut ops = Vec::new();

        ops.push(Op::StartTextSection);
        ops.push(Op::SetFont {
            font: PdfFontHandle::Builtin(font),
            size: Pt(font_size),
        });

        let mut y = page_h.0 - margin_y;

        for line in page_text.lines() {
            ops.push(Op::SetTextMatrix {
                matrix: TextMatrix::Translate(Pt::from(Mm(margin_x)), Pt::from(Mm(y))),
            });

            ops.push(Op::ShowText {
                items: vec![TextItem::Text(line.into())],
            });

            y -= line_h;
        }

        ops.push(Op::EndTextSection);

        pdf.pages.push(PdfPage::new(page_w, page_h, ops));
    }

    let mut warnings = Vec::new();
    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    Ok(bytes)
}
