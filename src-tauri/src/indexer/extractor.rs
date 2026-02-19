use std::path::Path;

use calamine::Reader;

use crate::error::{GhostError, Result};

/// Extract text content from a file based on its extension.
pub fn extract_text(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "txt" | "md" | "markdown" | "rst" | "csv" | "json" | "yaml" | "yml" | "toml" | "xml"
        | "html" | "htm" | "log" | "ini" | "cfg" | "conf" => extract_plain_text(path),
        "pdf" => extract_pdf(path),
        "docx" => extract_docx(path),
        "xlsx" | "xls" | "ods" => extract_spreadsheet(path),
        _ => Err(GhostError::Indexer(format!(
            "Unsupported file type: {}",
            extension
        ))),
    }
}

/// Check if a file extension is supported for indexing.
pub fn is_supported_extension(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "txt"
            | "md"
            | "markdown"
            | "rst"
            | "csv"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "xml"
            | "html"
            | "htm"
            | "log"
            | "ini"
            | "cfg"
            | "conf"
            | "pdf"
            | "docx"
            | "xlsx"
            | "xls"
            | "ods"
    )
}

fn extract_plain_text(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        GhostError::Indexer(format!("Failed to read {}: {}", path.display(), e))
    })?;
    Ok(content)
}

fn extract_pdf(path: &Path) -> Result<String> {
    let doc = lopdf::Document::load(path).map_err(|e| {
        GhostError::Indexer(format!("Failed to parse PDF {}: {}", path.display(), e))
    })?;

    let mut text = String::new();
    let pages = doc.get_pages();

    for page_num in pages.keys() {
        if let Ok(content) = doc.extract_text(&[*page_num]) {
            text.push_str(&content);
            text.push('\n');
        }
    }

    if text.trim().is_empty() {
        return Err(GhostError::Indexer(format!(
            "No extractable text in PDF: {}",
            path.display()
        )));
    }

    Ok(text)
}

/// Extract text from a DOCX file (ZIP archive containing XML).
fn extract_docx(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path).map_err(|e| {
        GhostError::Indexer(format!("Failed to open DOCX {}: {}", path.display(), e))
    })?;

    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|e| {
        GhostError::Indexer(format!("Failed to read DOCX ZIP {}: {}", path.display(), e))
    })?;

    let mut text = String::new();

    // DOCX stores text content in word/document.xml
    if let Ok(mut document) = archive.by_name("word/document.xml") {
        let mut xml_content = String::new();
        std::io::Read::read_to_string(&mut document, &mut xml_content).map_err(|e| {
            GhostError::Indexer(format!("Failed to read document.xml: {}", e))
        })?;

        // Extract text between <w:t> and <w:t xml:space="preserve"> tags
        let mut in_text_tag = false;
        let mut chars = xml_content.chars().peekable();
        let mut current_tag = String::new();

        while let Some(ch) = chars.next() {
            if ch == '<' {
                if in_text_tag {
                    in_text_tag = false;
                }
                current_tag.clear();
                current_tag.push(ch);
                // Read until >
                for next_ch in chars.by_ref() {
                    current_tag.push(next_ch);
                    if next_ch == '>' {
                        break;
                    }
                }
                // Check for paragraph/line break tags
                if current_tag.starts_with("<w:p ")
                    || current_tag.starts_with("<w:p>")
                    || current_tag == "</w:p>"
                {
                    if !text.is_empty() && !text.ends_with('\n') {
                        text.push('\n');
                    }
                } else if current_tag.starts_with("<w:br") {
                    text.push('\n');
                } else if current_tag.starts_with("<w:tab") {
                    text.push('\t');
                }

                // Check if this is an opening <w:t> tag
                if current_tag.starts_with("<w:t>") || current_tag.starts_with("<w:t ") {
                    in_text_tag = true;
                }
            } else if in_text_tag {
                text.push(ch);
            }
        }
    }

    if text.trim().is_empty() {
        return Err(GhostError::Indexer(format!(
            "No extractable text in DOCX: {}",
            path.display()
        )));
    }

    Ok(text)
}

fn extract_spreadsheet(path: &Path) -> Result<String> {
    let mut workbook: calamine::Sheets<std::io::BufReader<std::fs::File>> =
        calamine::open_workbook_auto(path).map_err(|e| {
            GhostError::Indexer(format!(
                "Failed to open spreadsheet {}: {}",
                path.display(),
                e
            ))
        })?;

    let mut text = String::new();
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    for name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&name) {
            text.push_str(&format!("--- Sheet: {} ---\n", name));
            for row in range.rows() {
                let row_text: Vec<String> = row
                    .iter()
                    .map(|cell: &calamine::Data| format!("{}", cell))
                    .collect();
                text.push_str(&row_text.join("\t"));
                text.push('\n');
            }
        }
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_plain_text() {
        let dir = std::env::temp_dir().join("ghost_test_extract");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "Hello, this is a test document.").unwrap();

        let text = extract_text(&file_path).unwrap();
        assert!(text.contains("Hello"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_unsupported_extension() {
        let path = Path::new("/fake/file.exe");
        let result = extract_text(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_supported_extensions() {
        assert!(is_supported_extension("txt"));
        assert!(is_supported_extension("pdf"));
        assert!(is_supported_extension("md"));
        assert!(is_supported_extension("xlsx"));
        assert!(is_supported_extension("docx"));
        assert!(!is_supported_extension("exe"));
        assert!(!is_supported_extension("dll"));
    }
}
