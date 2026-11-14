//! Domain types shared across db, workers, and gui.
pub mod entry;
pub mod filter;
pub mod hash;

pub use entry::{ClipEntry, ContentKind, FileMetadata};
pub use filter::{DateRange, ListDensity, SearchFilter, ShortcutAction};
pub use hash::{dedup_key, hash_bytes, hash_paths, hash_text};
