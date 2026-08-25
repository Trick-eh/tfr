pub mod persistence;
pub mod ui_additions;

use std::{fs, path::Path};

use docx_rs::{
    DocumentChild, Paragraph, ParagraphChild, RunChild, TableCellContent, TableChild, TableRowChild,
};

pub fn extract_text<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" => Ok(fs::read_to_string(path)?),
        "md" | "markdown" => extract_markdown(path),
        "pdf" => extract_pdf(path),
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

fn extract_markdown(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let raw_md = fs::read_to_string(path)?;
    let mut clean_text = String::with_capacity(raw_md.len());

    for line in raw_md.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") || trimmed.starts_with("---") || trimmed.starts_with("![") {
            continue;
        }

        let line_without_prefix = trimmed
            .trim_start_matches('#')
            .trim_start_matches(['*', '-', '+'])
            .trim();

        let mut in_code = false;
        let mut chars = line_without_prefix.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '`' => in_code = !in_code,
                '*' | '_' => continue,
                '[' => continue,
                ']' => {
                    if chars.peek() == Some(&'(') {
                        chars.next();
                        while let Some(&inner) = chars.peek() {
                            chars.next();
                            if inner == ')' {
                                break;
                            }
                        }
                    }
                }
                _ => clean_text.push(ch),
            }
        }
        clean_text.push(' ');
    }

    Ok(clean_text)
}

fn extract_pdf(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let text = pdf_extract::extract_text(path)?;

    Ok(text)
}
