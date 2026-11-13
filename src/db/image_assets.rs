//! Application-managed asset directory for image entries.
//! PNGs are stored on disk keyed by content hash; the DB holds only the path.
use crate::config;
use crate::db::DbError;
use std::path::PathBuf;

pub fn dir() -> PathBuf {
    config::assets_dir()
}

/// Store PNG bytes under `<assets>/<hash[:2]>/<hash>.png`, returning the path.
/// Two-level sharding avoids huge single directories.
pub fn save_png(png: &[u8], hash: &str) -> Result<PathBuf, DbError> {
    let shard = &hash[..hash.len().min(2)];
    let folder = dir().join(shard);
    std::fs::create_dir_all(&folder)?;
    let path = folder.join(format!("{hash}.png"));
    if !path.exists() {
        std::fs::write(&path, png)?;
    }
    Ok(path)
}

/// Load a PNG by its stored (possibly relative) path.
pub fn load_png(rel: &str) -> Result<Vec<u8>, DbError> {
    let p = PathBuf::from(rel);
    let full = if p.is_absolute() { p } else { dir().join(p) };
    Ok(std::fs::read(full)?)
}

/// Delete asset files no longer referenced by any entry.
/// Called after a prune/clear transaction commits.
pub fn delete_orphans(conn: &rusqlite::Connection) -> Result<usize, DbError> {
    use std::collections::HashSet;
    let mut referenced: HashSet<String> = HashSet::new();
    let mut stmt = conn.prepare("SELECT image_path FROM entries WHERE image_path IS NOT NULL")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    for r in rows {
        referenced.insert(
            PathBuf::from(r?)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        );
    }
    let mut removed = 0usize;
    let root = dir();
    if !root.exists() {
        return Ok(0);
    }
    let shards = std::fs::read_dir(&root)?;
    for shard in shards.flatten() {
        if !shard.path().is_dir() {
            continue;
        }
        for file in std::fs::read_dir(shard.path())?.flatten() {
            let name = file.file_name().to_string_lossy().into_owned();
            if name.ends_with(".png")
                && !referenced.contains(&name)
                && std::fs::remove_file(file.path()).is_ok()
            {
                removed += 1;
            }
        }
    }
    Ok(removed)
}
