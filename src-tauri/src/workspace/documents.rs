//! Documents of the library in the database: the row, the cached extracted
//! text (so startup does not read every file again) and the membership
//! bookkeeping that goes with adding and removing one.

use rusqlite::{params, Connection, OptionalExtension};

use crate::context::ContextResource;
use crate::db::context as context_db;
use crate::db::workspace_migrations::now_ms;

use super::store;
use super::types::ItemKind;

/// The extracted text of a document as saved.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedText {
    pub text: String,
    pub token_count: i64,
    /// `ready` or `missing`.
    pub status: String,
    /// Page-numbered text for PDFs; empty for other document types and older PDFs.
    pub pages: Vec<String>,
}

fn err(what: &str, e: impl std::fmt::Display) -> String {
    format!("{what}: {e}")
}

/// The database row of a loaded document.
pub fn db_row(resource: &ContextResource) -> context_db::ContextResource {
    context_db::ContextResource {
        id: resource.id.clone(),
        name: resource.name.clone(),
        file_type: resource.file_type.clone(),
        file_path: resource.file_path.clone(),
        size_bytes: resource.size_bytes as i64,
        token_count: resource.token_count as i64,
        preview: resource.preview.clone(),
        loaded_at: resource.loaded_at.clone(),
    }
}

/// Saves a loaded document: its row and its text. With `workspace_id` it also
/// joins that workspace in the same step; without, it is unfiled.
pub fn add_document(
    conn: &Connection,
    resource: &ContextResource,
    text: &str,
    workspace_id: Option<&str>,
) -> Result<(), String> {
    add_document_with_pages(conn, resource, text, &[], workspace_id)
}

/// Saves a document and its optional page-level PDF text in one transaction.
pub fn add_document_with_pages(
    conn: &Connection,
    resource: &ContextResource,
    text: &str,
    pages: &[String],
    workspace_id: Option<&str>,
) -> Result<(), String> {
    let tx = crate::db::immediate_tx(conn).map_err(|e| err("Couldn't save the document", e))?;
    context_db::add_context_resource(&tx, &db_row(resource))
        .map_err(|e| err("Couldn't save the document", e))?;
    store_text(&tx, &resource.id, text, resource.token_count as i64)?;
    store_pages(&tx, &resource.id, pages)?;
    if let Some(ws) = workspace_id {
        store::add_item(&tx, ws, ItemKind::Document, &resource.id, None)?;
    }
    tx.commit()
        .map_err(|e| err("Couldn't save the document", e))
}

/// Saves an edited note: its row, its text, and drops the index rows of the
/// old text (the next pass reads the new one).
pub fn update_note(
    conn: &Connection,
    resource: &ContextResource,
    text: &str,
) -> Result<(), String> {
    let tx = crate::db::immediate_tx(conn).map_err(|e| err("Couldn't save the note", e))?;
    context_db::update_context_resource_text(&tx, &db_row(resource))
        .map_err(|e| err("Couldn't save the note", e))?;
    store_text(&tx, &resource.id, text, resource.token_count as i64)?;
    store_pages(&tx, &resource.id, &[])?;
    crate::db::rag::delete_chunks_by_file(&tx, &resource.id)?;
    tx.commit().map_err(|e| err("Couldn't save the note", e))
}

/// Removes a document everywhere: row, text, index chunks and every membership.
pub fn remove_document(conn: &Connection, id: &str) -> Result<(), String> {
    let tx = crate::db::immediate_tx(conn).map_err(|e| err("Couldn't remove the document", e))?;
    let _ = context_db::delete_context_resource(&tx, id);
    crate::db::rag::delete_chunks_by_file(&tx, id)?;
    tx.execute(
        "DELETE FROM document_text WHERE resource_id = ?1",
        params![id],
    )
    .map_err(|e| err("Couldn't remove the document", e))?;
    tx.execute(
        "DELETE FROM document_pages WHERE resource_id = ?1",
        params![id],
    )
    .map_err(|e| err("Couldn't remove the document", e))?;
    store::remove_memberships(&tx, ItemKind::Document, id)?;
    super::purge::purge_search_rows(&tx, "document", id)?;
    tx.commit()
        .map_err(|e| err("Couldn't remove the document", e))
}

/// Saves the extracted text of a document.
pub fn store_text(conn: &Connection, id: &str, text: &str, token_count: i64) -> Result<(), String> {
    // One linear pass when the text is stored: the index and every later prompt read what a person can see.
    let text = crate::prompt_data::strip_hidden(text);
    let text = text.as_str();
    conn.execute(
        "INSERT INTO document_text (resource_id, text, token_count, status, error, extracted_at)
         VALUES (?1, ?2, ?3, 'ready', NULL, ?4)
         ON CONFLICT(resource_id) DO UPDATE SET text = excluded.text, token_count = excluded.token_count,
            status = 'ready', error = NULL, extracted_at = excluded.extracted_at",
        params![id, text, token_count, now_ms()],
    )
    .map(|_| ())
    .map_err(|e| err("Couldn't save the document text", e))
}

/// Replaces cached page text. Page numbers are one-based and blank pages are
/// preserved so that the stored index continues to match the PDF itself.
pub fn store_pages(conn: &Connection, id: &str, pages: &[String]) -> Result<(), String> {
    conn.execute(
        "DELETE FROM document_pages WHERE resource_id = ?1",
        params![id],
    )
    .map_err(|e| err("Couldn't update the document pages", e))?;
    let mut stmt = conn
        .prepare_cached(
            "INSERT INTO document_pages (resource_id, page_number, text) VALUES (?1, ?2, ?3)",
        )
        .map_err(|e| err("Couldn't update the document pages", e))?;
    for (index, page) in pages.iter().enumerate() {
        stmt.execute(params![
            id,
            index as i64 + 1,
            crate::prompt_data::strip_hidden(page)
        ])
        .map_err(|e| err("Couldn't update the document pages", e))?;
    }
    Ok(())
}

fn saved_pages(conn: &Connection, id: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT text FROM document_pages WHERE resource_id = ?1 ORDER BY page_number")
        .map_err(|e| err("Couldn't read the document pages", e))?;
    let rows = stmt
        .query_map(params![id], |row| row.get::<_, String>(0))
        .map_err(|e| err("Couldn't read the document pages", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| err("Couldn't read the document pages", e))
}

/// The document could not be read: its row stays and shows as missing.
pub fn mark_missing(conn: &Connection, id: &str, reason: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO document_text (resource_id, text, token_count, status, error, extracted_at)
         VALUES (?1, '', 0, 'missing', ?2, ?3)
         ON CONFLICT(resource_id) DO UPDATE SET text = '', token_count = 0, status = 'missing',
            error = excluded.error, extracted_at = excluded.extracted_at",
        params![id, reason, now_ms()],
    )
    .map(|_| ())
    .map_err(|e| err("Couldn't mark the document", e))?;
    store_pages(conn, id, &[])
}

/// The saved text of a document, if there is one.
pub fn saved_text(conn: &Connection, id: &str) -> Result<Option<SavedText>, String> {
    conn.query_row(
        "SELECT text, token_count, status FROM document_text WHERE resource_id = ?1",
        params![id],
        |row| {
            Ok(SavedText {
                text: row.get(0)?,
                token_count: row.get(1)?,
                status: row.get(2)?,
                pages: Vec::new(),
            })
        },
    )
    .optional()
    .map_err(|e| err("Couldn't read the document text", e))?
    .map(|mut saved| {
        saved.pages = saved_pages(conn, id)?;
        Ok(saved)
    })
    .transpose()
}

/// Saved texts of many documents at once (startup).
pub fn saved_texts(
    conn: &Connection,
) -> Result<std::collections::HashMap<String, SavedText>, String> {
    let mut stmt = conn
        .prepare("SELECT resource_id, text, token_count, status FROM document_text")
        .map_err(|e| err("Couldn't read the document text", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                SavedText {
                    text: row.get(1)?,
                    token_count: row.get(2)?,
                    status: row.get(3)?,
                    pages: Vec::new(),
                },
            ))
        })
        .map_err(|e| err("Couldn't read the document text", e))?;
    let mut saved: std::collections::HashMap<String, SavedText> = rows
        .collect::<Result<_, _>>()
        .map_err(|e| err("Couldn't read the document text", e))?;
    for (id, text) in &mut saved {
        text.pages = saved_pages(conn, id)?;
    }
    Ok(saved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::test_support::*;

    fn resource(id: &str) -> ContextResource {
        ContextResource {
            id: id.into(),
            name: format!("{id}.txt"),
            file_type: "txt".into(),
            file_path: "x".into(),
            size_bytes: 3,
            token_count: 2,
            preview: "p".into(),
            loaded_at: "now".into(),
            status: "ready".into(),
        }
    }

    #[test]
    fn a_document_added_in_a_workspace_joins_it_and_one_added_alone_is_unfiled() {
        let conn = memory_db();
        let a = new_ws(&conn, "A", false, false);
        add_document(&conn, &resource("in-a"), "alpha text", Some(&a)).unwrap();
        add_document(&conn, &resource("alone"), "beta text", None).unwrap();
        let members = store::memberships(&conn).unwrap();
        assert_eq!(
            members.get(&(ItemKind::Document, "in-a".to_string())),
            Some(&vec![a])
        );
        assert!(!members.contains_key(&(ItemKind::Document, "alone".to_string())));
        assert_eq!(
            saved_text(&conn, "in-a").unwrap().unwrap().text,
            "alpha text"
        );
        assert_eq!(saved_texts(&conn).unwrap().len(), 2);
    }

    #[test]
    fn page_text_is_saved_in_order_and_removed_with_the_document() {
        let conn = memory_db();
        let pages = vec![
            "first page".to_string(),
            String::new(),
            "third page".to_string(),
        ];
        add_document_with_pages(
            &conn,
            &resource("pdf"),
            "first page\nthird page",
            &pages,
            Some("general"),
        )
        .unwrap();
        assert_eq!(saved_text(&conn, "pdf").unwrap().unwrap().pages, pages);
        remove_document(&conn, "pdf").unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM document_pages WHERE resource_id = 'pdf'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn an_edited_note_changes_its_row_and_text() {
        let conn = memory_db();
        let mut r = resource("n");
        r.file_type = "note".into();
        add_document(&conn, &r, "before", Some("general")).unwrap();
        r.name = "Renamed".into();
        r.token_count = 7;
        update_note(&conn, &r, "after").unwrap();
        let row = context_db::get_context_resource(&conn, "n").unwrap();
        assert_eq!((row.name.as_str(), row.token_count), ("Renamed", 7));
        let saved = saved_text(&conn, "n").unwrap().unwrap();
        assert_eq!(
            (
                saved.text.as_str(),
                saved.token_count,
                saved.status.as_str()
            ),
            ("after", 7, "ready")
        );
    }

    #[test]
    fn a_failed_read_keeps_the_row_as_missing_and_removal_clears_everything() {
        let conn = memory_db();
        add_document(&conn, &resource("d"), "text", Some("general")).unwrap();
        mark_missing(&conn, "d", "the file was moved").unwrap();
        let saved = saved_text(&conn, "d").unwrap().unwrap();
        assert_eq!(
            (saved.status.as_str(), saved.text.as_str()),
            ("missing", "")
        );
        let rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM context_resources WHERE id = 'd'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(rows, 1);
        remove_document(&conn, "d").unwrap();
        assert!(saved_text(&conn, "d").unwrap().is_none());
        assert!(store::memberships(&conn).unwrap().is_empty());
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM context_resources", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 0);
    }
}
