//! SQLite storage, migrations, search, and image asset management.
pub mod entries;
pub mod image_assets;
pub mod schema;
pub mod search;
pub mod store;

pub use store::Store;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}
