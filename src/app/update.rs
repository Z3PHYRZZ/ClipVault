//! Pure, non-blocking `update`. All DB/clipboard work is dispatched to workers.
use crate::app::{AppState, Message};
use crate::domain::ContentKind;
use crate::platform;
use crate::workers::{DbCommand, DbResult, TrayAction, WorkerEvent};
use iced::Task;
use std::sync::atomic::Ordering;

/// Re-run the current search after any mutation/filter change.
fn refresh_search(state: &AppState) {
    let _ = state.db_tx.send(DbCommand::Search(state.filter.clone()));
}

/// Native Windows backdrop (v0.1.6). Resolves the primary window id, then runs
/// a `Send + 'static` closure with its raw HWND. The DWM call happens inside
/// that closure; the resulting `BackdropKind` is mapped back into a Message so
/// `AppState` can record it and `view()` can pick transparent vs gradient root.
fn backdrop_task(dark: bool) -> Task<Message> {
    iced::window::get_oldest()
        .then(move |maybe_id| match maybe_id {
            Some(id) => iced::window::run_with_handle(id, move |handle| {
                use raw_window_handle::RawWindowHandle;
                let hwnd = match handle.as_raw() {
                    RawWindowHandle::Win32(w) => w.hwnd.get(),
                    _ => 0,
                };
                if hwnd == 0 {
                    return crate::platform::backdrop::BackdropKind::None;
                }
                match crate::platform::backdrop::apply(hwnd, dark) {
                    Ok(kind) => kind,
                    Err(e) => {
                        tracing::warn!("native backdrop unavailable: {e}");
                        crate::platform::backdrop::BackdropKind::None
                    }
                }
            }),
            None => Task::none(),
        })
        .map(Message::BackdropApplied)
}

/// Close the main Iced window so the application actually exits after worker
/// teardown. Without this, the event loop would keep running with dead workers.
fn quit_task() -> Task<Message> {
    iced::window::get_oldest().then(|maybe_id| {
        if let Some(id) = maybe_id {
            iced::window::close(id)
        } else {
            Task::none()
        }
    })
}

/// Tear down workers, optionally clearing history, then close the window.
fn quit(state: &mut AppState) -> Task<Message> {
    if state.settings.clear_on_exit {
        // FIFO channel: ClearAll is processed by the DB worker before Shutdown.
        let _ = state.db_tx.send(DbCommand::ClearAll);
    }
    state.handles.shutdown(&state.db_tx);
    quit_task()
}

/// Restore an entry to the OS clipboard (writes are done on a short-lived task).
fn restore(state: &AppState, id: i64) -> Task<Message> {
    let Some(entry) = state.entries.iter().find(|e| e.id == id).cloned() else {
        return Task::none();
    };
    Task::perform(
        async move {
            let clip = platform::clipboard();
            let r: Result<(), platform::PlatformError> = match entry.kind {
                ContentKind::Text => clip.write_text(entry.text_content.as_deref().unwrap_or("")),
                ContentKind::Image => entry
                    .image_path
                    .as_deref()
                    .and_then(|p| std::fs::read(p).ok())
                    .ok_or_else(|| platform::PlatformError::Other("image missing".into()))
                    .and_then(|png| clip.write_image(&png)),
                ContentKind::Files => {
                    let paths: Vec<String> = entry
                        .file_paths_json
                        .as_deref()
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or_default();
                    clip.write_files(&paths)
                }
            };
            r
        },
        |r: Result<(), platform::PlatformError>| match r {
            Ok(_) => Message::Noop,
            Err(e) => Message::Worker(Box::new(WorkerEvent::Error(format!("restore: {e}")))),
        },
    )
}

/// Apply a show/hide to the main window. `get_oldest()` resolves the primary
/// window id; on show we also restore focus (the point of a global hotkey).
fn window_task(hide: bool) -> Task<Message> {
    iced::window::get_oldest().then(move |maybe_id| {
        if let Some(id) = maybe_id {
            if hide {
                iced::window::minimize(id, true)
            } else {
                Task::batch(vec![
                    iced::window::minimize(id, false),
                    iced::window::gain_focus(id),
                ])
            }
        } else {
            Task::none()
        }
    })
}

pub fn update(state: &mut AppState, msg: Message) -> Task<Message> {
    match msg {
        Message::Noop => Task::none(),
        Message::ShowWindow => {
            state.window_hidden = false;
            window_task(false)
        }
        Message::HideWindow => {
            state.window_hidden = true;
            window_task(true)
        }

        // ---- worker results ----
        Message::Worker(ev) => match *ev {
            WorkerEvent::NewClipboard(payload) => {
                let _ = state.db_tx.send(DbCommand::Insert(payload));
                // Refresh is triggered by the DbResult::Done("insert") below.
                Task::none()
            }
            WorkerEvent::Db(result) => match result {
                DbResult::Entries(rows) => {
                    state.entries = rows;
                    Task::none()
                }
                DbResult::Count(n) => {
                    state.total_count = n;
                    Task::none()
                }
                DbResult::Entry(Some(e)) => {
                    // Load image preview bytes when an image entry is selected.
                    if e.kind == ContentKind::Image {
                        if let Some(p) = e.image_path.as_deref() {
                            state.preview_bytes = crate::db::image_assets::load_png(p).ok();
                        }
                    }
                    Task::none()
                }
                DbResult::Entry(None) => Task::none(),
                DbResult::Exported(path) => {
                    state.status = format!("Exported history to {path}");
                    Task::none()
                }
                DbResult::Imported(n) => {
                    state.status = format!("Imported {n} entries");
                    refresh_search(state);
                    Task::none()
                }
                DbResult::Done(what) => {
                    state.status = what.clone();
                    if what == "insert"
                        || what == "delete"
                        || what == "clear"
                        || what == "tags"
                        || what.starts_with("pruned")
                        || what == "pin"
                        || what == "favorite"
                        || what.starts_with("deleted ")
                        || what.starts_with("pinned ")
                        || what.starts_with("favorited ")
                    {
                        refresh_search(state);
                    }
                    Task::none()
                }
                DbResult::Error(e) => {
                    state.status = format!("Error: {e}");
                    Task::none()
                }
            },
            WorkerEvent::Hotkey(crate::domain::ShortcutAction::ShowHide) => {
                let hide = !state.window_hidden;
                state.window_hidden = hide;
                window_task(hide)
            }
            WorkerEvent::Hotkey(crate::domain::ShortcutAction::QuickPaste(n)) => {
                // Quick-paste 1..9 maps to the nth row in the current (paged,
                // filtered) history. Out-of-range indices are a no-op.
                let idx = n.saturating_sub(1) as usize;
                if let Some(entry) = state.entries.get(idx) {
                    let id = entry.id;
                    restore(state, id)
                } else {
                    Task::none()
                }
            }
            WorkerEvent::Tray(TrayAction::Show) => {
                state.window_hidden = false;
                window_task(false)
            }
            WorkerEvent::Tray(TrayAction::Settings) => {
                state.show_settings = true;
                Task::none()
            }
            WorkerEvent::Tray(TrayAction::Pause | TrayAction::Resume) => {
                let paused = !state.paused;
                state.paused = paused;
                state.paused_flag.store(paused, Ordering::SeqCst);
                Task::none()
            }
            WorkerEvent::Tray(TrayAction::Quit) => quit(state),
            WorkerEvent::Error(e) => {
                state.status = format!("Error: {e}");
                Task::none()
            }
        },

        // ---- search / filters ----
        Message::SearchChanged(q) => {
            state.filter.query = q;
            state.filter.page = 0;
            refresh_search(state);
            Task::none()
        }
        Message::FilterKind(k) => {
            state.filter.kind = k;
            state.filter.page = 0;
            refresh_search(state);
            Task::none()
        }
        Message::FilterDate(d) => {
            state.filter.date = d;
            state.filter.page = 0;
            refresh_search(state);
            Task::none()
        }
        Message::TogglePinnedOnly(v) => {
            state.filter.pinned_only = v;
            state.filter.page = 0;
            refresh_search(state);
            Task::none()
        }
        Message::ToggleFavoritesOnly(v) => {
            state.filter.favorites_only = v;
            state.filter.page = 0;
            refresh_search(state);
            Task::none()
        }
        Message::ClearFilters => {
            state.filter.query.clear();
            state.filter.kind = None;
            state.filter.date = crate::domain::DateRange::All;
            state.filter.pinned_only = false;
            state.filter.favorites_only = false;
            state.filter.tag = None;
            state.filter.page = 0;
            refresh_search(state);
            Task::none()
        }
        // Hover is pure state: styles recompute on the next frame, giving a
        // smooth lift/border animation without a blocking timer.
        Message::HoverCard(id) => {
            state.hovered_card = id;
            Task::none()
        }
        Message::HoverNav(n) => {
            state.hovered_nav = n;
            Task::none()
        }

        // ---- entry actions ----
        Message::SelectEntry(id) => {
            state.selected = Some(id);
            // Seed the tag editor draft from the currently displayed row.
            state.tags_input = state
                .entries
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.tags().join(", "))
                .unwrap_or_default();
            let _ = state.db_tx.send(DbCommand::Get(id));
            Task::none()
        }
        Message::PinEntry(id, v) => {
            let _ = state.db_tx.send(DbCommand::SetPin(id, v));
            Task::none()
        }
        Message::FavoriteEntry(id, v) => {
            let _ = state.db_tx.send(DbCommand::SetFavorite(id, v));
            Task::none()
        }
        Message::DeleteEntry(id) => {
            if state.selected == Some(id) {
                state.selected = None;
            }
            let _ = state.db_tx.send(DbCommand::Delete(id));
            Task::none()
        }
        Message::RequestClearAll => {
            state.show_clear_confirm = true;
            Task::none()
        }
        Message::CancelClearAll => {
            state.show_clear_confirm = false;
            Task::none()
        }
        Message::ConfirmClearAll => {
            state.show_clear_confirm = false;
            state.selected = None;
            let _ = state.db_tx.send(DbCommand::ClearAll);
            Task::none()
        }
        Message::RestoreEntry(id) => restore(state, id),
        Message::OpenPreviewInExplorer(_id) => Task::none(),

        // ---- v0.1.4 organization tools ----
        Message::FilterTag(tag) => {
            state.filter.tag = tag;
            state.filter.page = 0;
            refresh_search(state);
            Task::none()
        }
        Message::SetTags(id, tags) => {
            let _ = state.db_tx.send(DbCommand::SetTags(id, tags));
            Task::none()
        }
        Message::TagsDraftChanged(s) => {
            state.tags_input = s;
            Task::none()
        }
        Message::ApplyTags => {
            if let Some(id) = state.selected {
                let tags: Vec<String> = state
                    .tags_input
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
                let _ = state.db_tx.send(DbCommand::SetTags(id, tags));
            }
            Task::none()
        }
        Message::ToggleSelect(id) => {
            if !state.selected_ids.remove(&id) {
                state.selected_ids.insert(id);
            }
            Task::none()
        }
        Message::SelectAll => {
            state
                .selected_ids
                .extend(state.entries.iter().map(|e| e.id));
            Task::none()
        }
        Message::ClearSelection => {
            state.selected_ids.clear();
            state.show_delete_selected_confirm = false;
            Task::none()
        }
        Message::RequestDeleteSelected => {
            if !state.selected_ids.is_empty() {
                state.show_delete_selected_confirm = true;
            }
            Task::none()
        }
        Message::ConfirmDeleteSelected => {
            state.show_delete_selected_confirm = false;
            let ids: Vec<i64> = state.selected_ids.drain().collect();
            state.selected = None;
            let _ = state.db_tx.send(DbCommand::DeleteMany(ids));
            Task::none()
        }
        Message::PinSelected(v) => {
            let ids: Vec<i64> = state.selected_ids.iter().copied().collect();
            if !ids.is_empty() {
                let _ = state.db_tx.send(DbCommand::SetPinMany(ids, v));
            }
            Task::none()
        }
        Message::FavoriteSelected(v) => {
            let ids: Vec<i64> = state.selected_ids.iter().copied().collect();
            if !ids.is_empty() {
                let _ = state.db_tx.send(DbCommand::SetFavoriteMany(ids, v));
            }
            Task::none()
        }
        Message::ExportHistory => {
            let path = crate::config::data_dir()
                .join("clipvault-export.json")
                .to_string_lossy()
                .into_owned();
            let _ = state.db_tx.send(DbCommand::ExportAll(path));
            Task::none()
        }
        Message::ImportHistory => {
            let path = crate::config::data_dir()
                .join("clipvault-export.json")
                .to_string_lossy()
                .into_owned();
            let _ = state.db_tx.send(DbCommand::ImportAll(path));
            Task::none()
        }
        Message::SetListDensity(d) => {
            state.settings.list_density = d;
            let _ = state.settings.save();
            Task::none()
        }
        Message::SetMaxEntriesKind(kind, n) => {
            match kind {
                crate::domain::ContentKind::Text => state.settings.max_entries_text = n,
                crate::domain::ContentKind::Image => state.settings.max_entries_image = n,
                crate::domain::ContentKind::Files => state.settings.max_entries_files = n,
            }
            let _ = state.settings.save();
            Task::none()
        }

        // ---- paging ----
        Message::LoadMore | Message::NextPage => {
            state.filter.page += 1;
            refresh_search(state);
            Task::none()
        }
        Message::PrevPage => {
            state.filter.page = state.filter.page.saturating_sub(1);
            refresh_search(state);
            Task::none()
        }

        // ---- settings ----
        Message::OpenSettings => {
            state.show_settings = true;
            Task::none()
        }
        Message::CloseSettings => {
            state.show_settings = false;
            let _ = state.settings.save();
            Task::none()
        }
        Message::SetTheme(t) => {
            state.settings.theme = t;
            let _ = state.settings.save();
            // Re-apply the DWM immersive-dark flag so Mica matches the new theme.
            backdrop_task(matches!(t, crate::config::ThemeMode::Dark))
        }
        Message::ApplyBackdrop => {
            backdrop_task(matches!(
                state.settings.theme,
                crate::config::ThemeMode::Dark
            ))
        }
        Message::BackdropApplied(kind) => {
            state.backdrop = kind;
            state.status = format!("Backdrop: {}", kind.label());
            Task::none()
        }
        }
        Message::SetMaxEntries(n) => {
            state.settings.max_entries = n;
            let _ = state.settings.save();
            Task::none()
        }
        Message::SetMaxAgeDays(d) => {
            state.settings.max_age_days = d;
            let _ = state.settings.save();
            Task::none()
        }
        Message::SetPauseMonitoring(v) => {
            state.settings.pause_monitoring = v;
            state.paused = v;
            state.paused_flag.store(v, Ordering::SeqCst);
            let _ = state.settings.save();
            Task::none()
        }
        Message::SetClearOnExit(v) => {
            state.settings.clear_on_exit = v;
            let _ = state.settings.save();
            Task::none()
        }
        Message::SetStartMinimized(v) => {
            state.settings.start_minimized = v;
            let _ = state.settings.save();
            Task::none()
        }
        Message::SetLaunchAtStartup(v) => {
            state.settings.launch_at_startup = v;
            let _ = platform::set_launch_at_startup(v);
            let _ = state.settings.save();
            Task::none()
        }
        Message::RebindShortcut(_action, s) => {
            state.settings.show_hotkey = s;
            let _ = state.settings.save();
            Task::none()
        }

        // ---- dialogs / window / retention ----
        Message::ToggleAbout => {
            state.show_about = !state.show_about;
            Task::none()
        }
        Message::Quit => quit(state),
        Message::RetentionTick => {
            let s = &state.settings;
            let _ = state.db_tx.send(DbCommand::Prune {
                max_entries: s.max_entries,
                max_age_days: s.max_age_days,
                caps: crate::db::bulk::TypeCaps {
                    text: s.max_entries_text,
                    image: s.max_entries_image,
                    files: s.max_entries_files,
                },
            });
            Task::none()
        }
    }
}
