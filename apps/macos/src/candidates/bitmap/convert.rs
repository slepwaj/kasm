//! 壳的帧类型 → 渲染器的帧类型。两边字段一一对应，spike 定型后壳直接用渲染器的类型，这层就没了。

use crate::candidates::frame::Frame;
use crate::candidates::preedit::{Preedit, PreeditStyle};
use crate::candidates::row::{Row, Tone};

pub(super) fn frame(frame: &Frame) -> kasm_render::Frame {
    kasm_render::Frame {
        preedit: frame.preedit.as_ref().map(preedit),
        rows: frame.rows.iter().map(row).collect(),
        highlighted: Some(frame.highlighted),
        columns: frame.columns,
        column_ems: frame.column_ems.clone(),
        footer: frame.footer.clone(),
        previous_page: frame.pages_around.0,
        next_page: frame.pages_around.1,
        sentence: frame.sentence.clone(),
        status: frame.status.clone(),
        mode: frame.mode.clone(),
    }
}

fn preedit(preedit: &Preedit) -> kasm_render::Preedit {
    kasm_render::Preedit {
        segments: preedit
            .segments
            .iter()
            .map(|segment| kasm_render::PreeditSegment {
                text: segment.text.clone(),
                style: match segment.style {
                    PreeditStyle::Typed => kasm_render::PreeditStyle::Typed,
                    PreeditStyle::Rest => kasm_render::PreeditStyle::Rest,
                    PreeditStyle::Struck => kasm_render::PreeditStyle::Struck,
                },
            })
            .collect(),
        cursor: preedit.cursor,
    }
}

fn row(row: &Row) -> kasm_render::Row {
    kasm_render::Row {
        index: row.index.clone(),
        text: row.text.clone(),
        code: row.code.clone(),
        annotation: row
            .annotation
            .iter()
            .map(|(text, tone)| {
                let tone = match tone {
                    Tone::Gloss => kasm_render::Tone::Gloss,
                    Tone::Fresh => kasm_render::Tone::Fresh,
                    Tone::Faint => kasm_render::Tone::Faint,
                    Tone::Pos => kasm_render::Tone::Pos,
                    Tone::Separator => kasm_render::Tone::Separator,
                };
                (text.clone(), tone)
            })
            .collect(),
        cloud: row.cloud,
    }
}
