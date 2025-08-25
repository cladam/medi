use crate::note::Note;
use lazy_static::lazy_static;
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, TantivyDocument};

// Define the schema for your search index.
// `lazy_static` ensures this is initialised only once.
lazy_static! {
    static ref SCHEMA: Schema = {
        let mut schema_builder = Schema::builder();
        // The key is stored and indexed so we can find it.
        schema_builder.add_text_field("key", STRING | STORED);
        // The title is indexed for searching.
        schema_builder.add_text_field("title", TEXT | STORED);
        // The content is the main searchable text.
        schema_builder.add_text_field("content", TEXT | STORED);
        // Tags are indexed as well.
        schema_builder.add_text_field("tags", TEXT | STORED);
        schema_builder.build()
    };
}

/// Opens an existing index or creates a new one.
pub fn open_index(path: &Path) -> Result<Index, tantivy::error::TantivyError> {
    std::fs::create_dir_all(path)?;
    let directory = MmapDirectory::open(path)?;
    let index = Index::open_or_create(directory, SCHEMA.clone())?;
    Ok(index)
}

/// Adds a single note to the search index.
/// This function is designed to be called within a re-indexing loop.
pub fn add_note_to_index(
    note: &Note,
    index_writer: &mut IndexWriter<tantivy::TantivyDocument>,
) -> Result<(), tantivy::error::TantivyError> {
    let schema = &SCHEMA;
    let key = schema.get_field("key")?;
    let title = schema.get_field("title")?;
    let content = schema.get_field("content")?;
    let tags_field = schema.get_field("tags")?;

    let mut doc = doc!(
        key => note.key.clone(),
        title => note.title.clone(),
        content => note.content.clone(),
    );

    for tag in &note.tags {
        doc.add_text(tags_field, tag);
    }

    index_writer.add_document(doc)?;
    Ok(())
}

/// Deletes a document from the index based on its key.
pub fn delete_note_from_index(
    key: &str,
    index_writer: &mut IndexWriter<tantivy::TantivyDocument>,
) -> Result<(), tantivy::error::TantivyError> {
    let schema = &SCHEMA;
    let key_field = schema.get_field("key")?;
    let key_term = Term::from_field_text(key_field, key);
    index_writer.delete_term(key_term);
    Ok(())
}

/// Searches the index for a query and returns a Vec of matching note keys.
pub fn search_notes(
    index: &Index,
    query_str: &str,
) -> Result<Vec<String>, tantivy::error::TantivyError> {
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;

    let searcher = reader.searcher();
    let key_field = SCHEMA.get_field("key")?;
    let title_field = SCHEMA.get_field("title")?;
    let content_field = SCHEMA.get_field("content")?;
    let tags_field = SCHEMA.get_field("tags")?;

    let query_parser = QueryParser::for_index(index, vec![title_field, content_field, tags_field]);
    let query = query_parser.parse_query(query_str)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let mut results = Vec::new();
    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        if let Some(value) = retrieved_doc.get_first(key_field) {
            if let Some(key_val) = value.as_str() {
                results.push(key_val.to_string());
            }
        }
    }

    Ok(results)
}
