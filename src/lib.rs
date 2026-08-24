pub mod persistence;
pub mod ui_additions;

use std::fs;
use std::path::Path;

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
        // "docx" => {
        //     let bytes = fs::read(path)?;
        //     let docx = docx_rs::read_docx(&bytes)?;
        //     Ok(docx.document.to_string())
        // }
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
