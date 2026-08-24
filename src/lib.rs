pub mod persistence;
pub mod ui_additions;

use docx_rs::*;
use std::{fs, path::Path};

pub fn extract_text<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" => Ok(fs::read_to_string(path)?),
        "pdf" => Ok(pdf_extract::extract_text(path)?),
        "docx" => {
            let bytes = fs::read(path)?;
            let docx = docx_rs::read_docx(&bytes)?;
            let mut extracted_text = String::new();

            for child in docx.document.children {
                match child {
                    DocumentChild::Paragraph(p) => {
                        extract_docx_paragraph_text(p.as_ref(), &mut extracted_text);
                        extracted_text.push('\n');
                    }
                    DocumentChild::Table(t) => {
                        for row in t.rows {
                            let TableChild::TableRow(tr) = row;
                            for cell in tr.cells {
                                let TableRowChild::TableCell(tc) = cell;
                                for cell_elem in tc.children {
                                    if let TableCellContent::Paragraph(p) = cell_elem {
                                        extract_docx_paragraph_text(
                                            p.as_ref(),
                                            &mut extracted_text,
                                        );
                                        extracted_text.push(' ')
                                    }
                                }
                            }
                            extracted_text.push('\n');
                        }
                    }
                    _ => {}
                }
            }

            Ok(extracted_text)
        }
        "epub" => {
            let mut doc = epub::doc::EpubDoc::new(path)?;
            let mut full_text = String::new();
            while doc.go_next() {
                if let Some((html, _)) = doc.get_current_str() {
                    let plain = html2text::from_read(html.as_bytes(), 80).unwrap();
                    full_text.push_str(&plain);
                    full_text.push(' ');
                }
            }
            Ok(full_text)
        }
        _ => Err("Unsupported file format".into()),
    }
}

fn extract_docx_paragraph_text(p: &Paragraph, out: &mut String) {
    for child in &p.children {
        if let ParagraphChild::Run(run) = child {
            for run_child in &run.children {
                if let RunChild::Text(t) = run_child {
                    out.push_str(&t.text);
                }
            }
        }
    }
}
