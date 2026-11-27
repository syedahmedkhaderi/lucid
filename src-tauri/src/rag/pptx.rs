//! Bounded extraction of readable text from PowerPoint OOXML slide parts.
//!
//! This extracts text and basic table rows from slide XML. It does not OCR images
//! or infer chart values; keeping slide numbers beside the text preserves a
//! useful citation location for the local retrieval pipeline.

use std::io::{Read, Seek};

const MAX_PPTX_ENTRIES: usize = 4_000;
const MAX_PPTX_SLIDES: usize = 500;
const MAX_PPTX_XML: u64 = 16 * 1024 * 1024;
const MAX_PPTX_XML_TOTAL: u64 = 128 * 1024 * 1024;
const MAX_PPTX_COMPRESSION_RATIO: u64 = 400;
const MAX_PPTX_TEXT: usize = 64 * 1024 * 1024;

/// Extract the text in slide order, labeling each block with its slide number.
pub fn extract_pptx_text(file_path: &str) -> Result<String, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open PowerPoint file '{}': {}", file_path, e))?;
    extract_pptx_reader(file)
}

/// Extract from PPTX bytes. Kept public for safe, deterministic parser tests.
pub fn extract_pptx_text_bytes(bytes: &[u8]) -> Result<String, String> {
    extract_pptx_reader(std::io::Cursor::new(bytes))
}

fn extract_pptx_reader<R: Read + Seek>(reader: R) -> Result<String, String> {
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| format!("Failed to read PowerPoint file as ZIP archive: {}", e))?;
    if archive.len() > MAX_PPTX_ENTRIES {
        return Err("This PowerPoint file has too many parts".to_string());
    }

    let mut slides = Vec::new();
    let mut total_xml = 0_u64;
    for index in 0..archive.len() {
        let (name, size) = {
            let entry = archive
                .by_index(index)
                .map_err(|e| format!("Failed to inspect PowerPoint part: {}", e))?;
            (
                entry
                    .name()
                    .map_err(|e| format!("Failed to inspect PowerPoint part name: {}", e))?
                    .to_string(),
                entry.size(),
            )
        };
        let Some(slide_number) = slide_number(&name) else {
            continue;
        };
        if size > MAX_PPTX_XML || total_xml.saturating_add(size) > MAX_PPTX_XML_TOTAL {
            return Err("This PowerPoint file expands to too much data".to_string());
        }
        total_xml += size;
        slides.push((slide_number, index));
        if slides.len() > MAX_PPTX_SLIDES {
            return Err("This PowerPoint file has too many slides".to_string());
        }
    }

    if slides.is_empty() {
        return Err("This PowerPoint file contains no readable slides".to_string());
    }
    slides.sort_by_key(|(number, _)| *number);

    let mut output = String::new();
    for (number, index) in slides {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("Failed to read PowerPoint slide {}: {}", number, e))?;
        let (size, compressed) = (entry.size(), entry.compressed_size().max(1));
        if size > 1024 * 1024 && size > compressed.saturating_mul(MAX_PPTX_COMPRESSION_RATIO) {
            return Err(format!("PowerPoint slide {} expands too much", number));
        }
        let mut xml = Vec::with_capacity(size.min(MAX_PPTX_XML) as usize);
        entry
            .by_ref()
            .take(MAX_PPTX_XML + 1)
            .read_to_end(&mut xml)
            .map_err(|e| format!("Failed to read PowerPoint slide {}: {}", number, e))?;
        if xml.len() as u64 > MAX_PPTX_XML {
            return Err(format!("PowerPoint slide {} expands too much", number));
        }
        let xml = std::str::from_utf8(&xml)
            .map_err(|_| format!("PowerPoint slide {} contains invalid UTF-8", number))?;
        let text = parse_slide_xml(xml)?;
        if text.trim().is_empty() {
            continue;
        }
        let block = format!("[Slide {number}]\n{text}\n\n");
        if output.len().saturating_add(block.len()) > MAX_PPTX_TEXT {
            return Err("Extracted PowerPoint text exceeds the supported size".to_string());
        }
        output.push_str(&block);
    }

    if output.trim().is_empty() {
        return Err("This PowerPoint file contains no readable slide text".to_string());
    }
    Ok(output.trim().to_string())
}

fn slide_number(name: &str) -> Option<u32> {
    name.strip_prefix("ppt/slides/slide")?
        .strip_suffix(".xml")?
        .parse()
        .ok()
}

fn parse_slide_xml(xml: &str) -> Result<String, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut result = String::new();
    let mut in_text = false;
    let mut in_table = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(tag)) => match tag.local_name().as_ref() {
                "t" => in_text = true,
                "tbl" => in_table = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                result.push_str(&text.xml10_content());
            }
            Ok(Event::CData(text)) if in_text => {
                result.push_str(&text.xml10_content());
            }
            Ok(Event::GeneralRef(reference)) if in_text => {
                if let Some(ch) = reference.resolve_char_ref().map_err(|e| {
                    format!("Invalid character reference in PowerPoint slide XML: {}", e)
                })? {
                    result.push(ch);
                } else {
                    match reference.as_ref() {
                        "amp" => result.push('&'),
                        "lt" => result.push('<'),
                        "gt" => result.push('>'),
                        "apos" => result.push('\''),
                        "quot" => result.push('"'),
                        other => {
                            return Err(format!(
                                "Unsupported XML entity in PowerPoint slide: &{other};"
                            ));
                        }
                    }
                }
            }
            Ok(Event::End(tag)) => {
                let name = tag.local_name();
                if name.as_ref() == "t" {
                    in_text = false;
                } else if name.as_ref() == "tbl" {
                    in_table = false;
                } else if name.as_ref() == "p" && !result.ends_with('\n') {
                    result.push('\n');
                } else if name.as_ref() == "tc" && in_table {
                    while result.ends_with('\n') {
                        result.pop();
                    }
                    result.push_str(" | ");
                } else if name.as_ref() == "tr" && in_table {
                    if result.ends_with(" | ") {
                        result.truncate(result.len() - 3);
                    }
                    result.push('\n');
                }
            }
            Ok(Event::Empty(tag)) => match tag.local_name().as_ref() {
                "br" => result.push('\n'),
                "tab" => result.push(' '),
                _ => {}
            },
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(format!("Invalid PowerPoint slide XML: {}", e)),
        }
    }

    let mut cleaned = String::with_capacity(result.len());
    let mut newlines = 0;
    for ch in result.trim().chars() {
        if ch == '\n' {
            newlines += 1;
            if newlines <= 2 {
                cleaned.push(ch);
            }
        } else {
            newlines = 0;
            cleaned.push(ch);
        }
    }
    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pptx(parts: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let mut buffer = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buffer);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (name, contents) in parts {
                zip.start_file(*name, options).unwrap();
                zip.write_all(contents).unwrap();
            }
            zip.finish().unwrap();
        }
        buffer.into_inner()
    }

    #[test]
    fn reads_slide_text_in_numeric_order_and_unescapes_xml() {
        let archive = pptx(&[
            ("ppt/slides/slide10.xml", br#"<p:sld><a:t>Last</a:t></p:sld>"#),
            ("ppt/slides/slide2.xml", br#"<p:sld><a:p><a:r><a:t>Cost &amp; delivery</a:t></a:r></a:p><a:p><a:r><a:t>Ready &lt; 30 days</a:t></a:r></a:p></p:sld>"#),
            ("ppt/presentation.xml", br#"<p:presentation/>"#),
        ]);
        let text = extract_pptx_text_bytes(&archive).unwrap();
        assert!(text.find("[Slide 2]").unwrap() < text.find("[Slide 10]").unwrap());
        assert!(text.contains("Cost & delivery"), "{text:?}");
        assert!(text.contains("Ready < 30 days"));
        assert!(text.contains("[Slide 10]\nLast"));
    }

    #[test]
    fn reads_table_cells_as_a_row() {
        let xml = r#"<p:sld><a:tbl><a:tr><a:tc><a:p><a:r><a:t>Part</a:t></a:r></a:p></a:tc><a:tc><a:p><a:r><a:t>Value</a:t></a:r></a:p></a:tc></a:tr></a:tbl></p:sld>"#;
        let parsed = parse_slide_xml(xml).unwrap();
        assert!(parsed.contains("Part | Value"), "{parsed:?}");
    }

    #[test]
    fn rejects_invalid_and_unreadable_archives() {
        assert!(extract_pptx_text_bytes(b"not a PowerPoint file")
            .unwrap_err()
            .contains("ZIP archive"));
        let archive = pptx(&[("ppt/presentation.xml", b"<p:presentation/>")]);
        assert!(extract_pptx_text_bytes(&archive)
            .unwrap_err()
            .contains("no readable slides"));
    }

    #[test]
    fn rejects_high_expansion_slide_xml() {
        let payload = format!("<p:sld><a:t>{}</a:t></p:sld>", "x".repeat(2 * 1024 * 1024));
        let archive = pptx(&[("ppt/slides/slide1.xml", payload.as_bytes())]);
        assert!(extract_pptx_text_bytes(&archive)
            .unwrap_err()
            .contains("expands too much"));
    }
}
