use lazy_static::lazy_static;
use std::path::Path;
use tantivy::directory::MmapDirectory;
use tantivy::schema::*;
use tantivy::{doc, Index};

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