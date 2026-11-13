//! The sole owner of the SQLite connection. Runs on its own thread.
use crate::db::{DbError, entries, image_assets, search, store::Store};
use crate::workers::{DbCommand, DbResult, WorkerEvent};

/// Command loop. `db_rx` receives GUI commands; results are posted to `events_tx`.
pub fn run(
    db_path: std::path::PathBuf,
    db_rx: flume::Receiver<DbCommand>,
    events_tx: flume::Sender<WorkerEvent>,
) {
    let store = match Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            let _ = events_tx.send(WorkerEvent::Error(format!("db open failed: {e}")));
            return;
        }
    };
    let conn = store.conn();

    while let Ok(cmd) = db_rx.recv() {
        let result = execute(conn, &cmd);
        match result {
            Ok(r) => {
                if matches!(&r, DbResult::Done(s) if s == "shutdown") {
                    let _ = store.checkpoint();
                    break;
                }
                let _ = events_tx.send(WorkerEvent::Db(r));
            }
            Err(e) => {
                let _ = events_tx.send(WorkerEvent::Db(DbResult::Error(e.to_string())));
            }
        }
    }
    let _ = store.checkpoint();
}

fn execute(conn: &rusqlite::Connection, cmd: &DbCommand) -> Result<DbResult, DbError> {
    match cmd {
        DbCommand::Insert(p) => {
            let _id = entries::insert_dedup(conn, p)?;
            Ok(DbResult::Done("insert".into()))
        }
        DbCommand::Search(f) => {
            let rows = search::search(conn, f)?;
            Ok(DbResult::Entries(rows))
        }
        DbCommand::Get(id) => {
            let e = entries::get(conn, *id)?;
            Ok(DbResult::Entry(e))
        }
        DbCommand::SetPin(id, v) => {
            entries::set_pin(conn, *id, *v)?;
            Ok(DbResult::Done("pin".into()))
        }
        DbCommand::SetFavorite(id, v) => {
            entries::set_favorite(conn, *id, *v)?;
            Ok(DbResult::Done("favorite".into()))
        }
        DbCommand::Delete(id) => {
            entries::delete(conn, *id)?;
            Ok(DbResult::Done("delete".into()))
        }
        DbCommand::ClearAll => {
            entries::clear_all(conn)?;
            let _ = image_assets::delete_orphans(conn);
            Ok(DbResult::Done("clear".into()))
        }
        DbCommand::Prune {
            max_entries,
            max_age_days,
        } => {
            let n = entries::prune(conn, *max_entries, *max_age_days)?;
            let _ = image_assets::delete_orphans(conn);
            Ok(DbResult::Done(format!("pruned {n}")))
        }
        DbCommand::Shutdown => Ok(DbResult::Done("shutdown".into())),
    }
}
