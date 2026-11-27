//! The indexer's inputs from the workspace data: documents (their saved
//! text), folders linked to a workspace's Library, talking points marked as AI
//! context. Each call reads the database fresh through its own connection. A
//! read that fails is an error (the pass then leaves the index as it is).

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use rusqlite::Connection;

use crate::sources::config::PersonalSourcesConfig;
use crate::workspace_search::indexer::{DocInput, FolderInput, IndexProvider, TalkingPointInput};

pub struct WorkspaceIndexProvider {
    pub db_path: PathBuf,
    pub sources: Arc<RwLock<PersonalSourcesConfig>>,
}

impl WorkspaceIndexProvider {
    fn conn(&self) -> Result<Connection, String> {
        crate::db::open_reader(&self.db_path)
            .map_err(|e| format!("Couldn't open the database: {e}"))
    }
}

fn read_err(what: &str, e: rusqlite::Error) -> String {
    format!("Couldn't read {what}: {e}")
}

impl IndexProvider for WorkspaceIndexProvider {
    fn documents(&self) -> Result<Vec<DocInput>, String> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT r.id, r.name, t.text, p.page_number, p.text, r.file_type, r.file_path FROM context_resources r
                 JOIN document_text t ON t.resource_id = r.id
                 LEFT JOIN document_pages p ON p.resource_id = r.id
                 WHERE t.status = 'ready' ORDER BY r.id, p.page_number",
            )
            .map_err(|e| read_err("the documents", e))?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| read_err("the documents", e))?;
        let mut docs: Vec<DocInput> = Vec::new();
        let mut by_id = std::collections::HashMap::new();
        let mut files = std::collections::HashMap::new();
        for row in rows {
            let (id, name, text, page_number, page_text, file_type, file_path) =
                row.map_err(|e| read_err("the documents", e))?;
            files.entry(id.clone()).or_insert((file_type, file_path));
            let index = *by_id.entry(id.clone()).or_insert_with(|| {
                let index = docs.len();
                docs.push(DocInput {
                    id,
                    name,
                    text,
                    pages: None,
                });
                index
            });
            if let (Some(page_number), Some(page_text)) = (page_number, page_text) {
                let pages = docs[index].pages.get_or_insert_with(Vec::new);
                let page_number = page_number as usize;
                if page_number > 0 {
                    pages.resize_with(page_number, String::new);
                    pages[page_number - 1] = page_text;
                }
            }
        }
        // Existing PDFs predate the page-text cache. Backfill them from their
        // copied file the next time the indexer reads the document list.
        let mut backfilled = Vec::new();
        for doc in &mut docs {
            if doc.pages.as_ref().is_some_and(|pages| !pages.is_empty()) {
                continue;
            }
            let Some((file_type, path)) = files.get(&doc.id) else {
                continue;
            };
            if file_type != "pdf" {
                continue;
            }
            if let Ok(pages) = crate::context::pdf_extractor::extract_pdf_pages_from_pdf(
                path,
                crate::workspace_search::passages::MAX_SOURCE_CHARS,
            ) {
                doc.pages = Some(pages.clone());
                backfilled.push((doc.id.clone(), pages));
            }
        }
        if !backfilled.is_empty() {
            if let Ok(writer) = crate::db::open_writer(&self.db_path) {
                if let Ok(tx) = crate::db::immediate_tx(&writer) {
                    let saved = backfilled.iter().try_for_each(|(id, pages)| {
                        crate::workspace::documents::store_pages(&tx, id, pages)
                    });
                    if saved.is_ok() {
                        let _ = tx.commit();
                    }
                }
            }
        }
        Ok(docs)
    }

    fn folders(&self) -> Result<Vec<FolderInput>, String> {
        let conn = self.conn()?;
        let linked: std::collections::HashSet<String> = {
            let mut stmt = conn
                .prepare("SELECT DISTINCT ref_id FROM workspace_items WHERE kind = 'folder'")
                .map_err(|e| read_err("the folders", e))?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| read_err("the folders", e))?;
            rows.collect::<Result<_, _>>()
                .map_err(|e| read_err("the folders", e))?
        };
        let cfg = self
            .sources
            .read()
            .map_err(|_| "Couldn't read your sources".to_string())?
            .clone();
        Ok(cfg
            .notes_folders
            .iter()
            .map(|f| FolderInput {
                id: f.id.clone(),
                path: PathBuf::from(&f.path),
                // a folder that is switched off in Settings stays off; one linked to a workspace is used
                index_enabled: f.enabled && linked.contains(&f.id),
            })
            .collect())
    }

    fn talking_points(&self) -> Result<Vec<TalkingPointInput>, String> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, title, plain_text, in_context FROM talking_points")
            .map_err(|e| read_err("the talking points", e))?;
        let rows = stmt
            .query_map([], |r| {
                // stored as written; what is indexed (and later quoted) has no hidden characters
                let (title, text): (String, String) = (r.get(1)?, r.get(2)?);
                Ok(TalkingPointInput {
                    id: r.get(0)?,
                    title: crate::prompt_data::strip_hidden(&title),
                    plain_text: crate::prompt_data::strip_hidden(&text),
                    in_context: r.get::<_, i64>(3)? != 0,
                })
            })
            .map_err(|e| read_err("the talking points", e))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| read_err("the talking points", e))
    }
}
