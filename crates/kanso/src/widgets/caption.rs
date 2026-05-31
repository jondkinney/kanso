//! Caption text with inline backtick→`code` pills.
//!
//! This is the apps' most elaborate hand-rolled widget (vernier
//! `prefs.rs:3056`, hyprcorrect's equivalent): muted explainer text that
//! parses `` `backtick` `` spans into monospace pills with a subtle fill
//! painted to hug the glyph metrics — not the full row height — so the
//! pill sits centered on the text the way a GitHub comment renders inline
//! code. Ported faithfully; only the literals are swapped for
//! [`crate::palette`] / [`crate::metrics`] tokens.

use egui::Ui;
use egui::text::LayoutJob;

use crate::{metrics, palette};

/// Render `text` as a muted caption, turning `` `backtick` `` spans into
/// inline code pills. Lays out at the live available width so wrapping is
/// correct, then hand-paints the pill backdrops.
pub fn caption(ui: &mut Ui, text: &str) {
    let line_height = metrics::CAPTION_LINE_HEIGHT;
    // Code spans get NO background from the layout — we paint our own
    // rects below at a tighter y range, centered on the glyphs.
    let plain = egui::TextFormat {
        font_id: egui::FontId::proportional(metrics::CAPTION_SIZE),
        color: palette::TEXT_MUTED,
        line_height: Some(line_height),
        valign: egui::Align::Center,
        ..Default::default()
    };
    let code = egui::TextFormat {
        font_id: egui::FontId::monospace(metrics::CAPTION_CODE_SIZE),
        color: palette::CODE_TEXT,
        line_height: Some(line_height),
        valign: egui::Align::Center,
        ..Default::default()
    };
    // No-break space inside the pill so the backdrop extends a glyph-
    // width past the code text without exposing a wrap opportunity.
    const NBSP: char = '\u{00A0}';
    let mut job = LayoutJob::default();
    job.wrap.max_width = ui.available_width();
    let mut in_code = false;
    let mut buf = String::new();
    let flush = |job: &mut LayoutJob, buf: &mut String, in_code: bool| {
        if buf.is_empty() {
            return;
        }
        if in_code {
            job.append(&format!("{NBSP}{buf}{NBSP}"), 0.0, code.clone());
        } else {
            job.append(buf, 0.0, plain.clone());
        }
        buf.clear();
    };
    for c in text.chars() {
        if c == '`' {
            flush(&mut job, &mut buf, in_code);
            in_code = !in_code;
        } else {
            buf.push(c);
        }
    }
    flush(&mut job, &mut buf, in_code);

    // Lay out, allocate, then paint code backdrops manually so we can
    // use a tighter y-range than the full row height.
    let galley = ui.fonts(|f| f.layout_job(job));
    let (rect, _resp) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
    let origin = rect.min;
    let painter = ui.painter();

    let bg_x_slop: f32 = 1.0;
    let bg_y_pad_top: f32 = 2.0;
    let bg_y_pad_bot: f32 = 1.0;
    // A code run carries its own y extent so the backdrop hugs the
    // actual glyph metrics rather than the full row line height.
    type CodeRun = (f32, f32, f32, f32); // (x0, x1, y_min, y_max)
    let mut in_code_run = false;
    for row in &galley.rows {
        let row_rect = row.rect();
        let mut run: Option<CodeRun> = None;
        let flush_run = |run: &mut Option<CodeRun>| {
            if let Some((x0, x1, y_min, y_max)) = run.take()
                && y_min.is_finite()
                && y_max.is_finite()
            {
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(x0 + bg_x_slop, y_min - bg_y_pad_top),
                        egui::pos2(x1 - bg_x_slop, y_max + bg_y_pad_bot),
                    ),
                    metrics::CODE_CORNER,
                    palette::CODE_BG,
                );
            }
        };
        for glyph in &row.glyphs {
            let x0 = origin.x + row_rect.min.x + glyph.pos.x;
            let size = glyph.size();
            let x1 = x0 + size.x;
            // glyph.pos.y is the BASELINE within the row; build the
            // visible glyph y-extent from baseline + font metrics so the
            // backdrop hugs ascender→descender, not the full line height.
            let baseline = origin.y + row_rect.min.y + glyph.pos.y;
            let gy_min = baseline - glyph.font_ascent;
            let gy_max = baseline + (glyph.font_height - glyph.font_ascent);
            if glyph.chr == NBSP {
                // NBSP marks the start / end of a code span; its own
                // y-extent is tiny so don't fold it into the run bounds.
                match run {
                    Some((_, ref mut x_end, _, _)) => *x_end = x1,
                    None => run = Some((x0, x1, f32::INFINITY, f32::NEG_INFINITY)),
                }
                if in_code_run {
                    in_code_run = false;
                    flush_run(&mut run);
                } else {
                    in_code_run = true;
                }
            } else if in_code_run {
                match run {
                    Some((_, ref mut x_end, ref mut y_min, ref mut y_max)) => {
                        *x_end = x1;
                        if gy_min < *y_min {
                            *y_min = gy_min;
                        }
                        if gy_max > *y_max {
                            *y_max = gy_max;
                        }
                    }
                    None => run = Some((x0, x1, gy_min, gy_max)),
                }
            }
        }
        flush_run(&mut run);
    }

    painter.galley(origin, galley, egui::Color32::PLACEHOLDER);
}
