//! Graceful shutdown handles for Win32 worker threads.
//!
//! Each platform subsystem (clipboard listener, hotkeys, tray) owns a thread
//! running a `GetMessageW` loop. To stop it cleanly we post `WM_QUIT` to that
//! thread's message queue (not to a window) and then join. This is idempotent
//! and `Drop`-safe, so forgetting an explicit shutdown still joins the thread.

#[cfg(windows)]
mod imp {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT};

    /// Owns a Win32 worker thread's id and join handle.
    pub struct ThreadHandle {
        pub thread_id: u32,
        join: Option<std::thread::JoinHandle<()>>,
    }

    impl ThreadHandle {
        pub fn new(thread_id: u32, join: std::thread::JoinHandle<()>) -> Self {
            Self {
                thread_id,
                join: Some(join),
            }
        }

        /// Capture the current OS thread id (useful when a subsystem reports its
        /// own id over a channel after spawning).
        pub fn from_current_thread(join: std::thread::JoinHandle<()>) -> Self {
            let thread_id = unsafe { GetCurrentThreadId() };
            Self::new(thread_id, join)
        }

        /// Post WM_QUIT then join. Safe to call more than once.
        pub fn shutdown(&mut self) {
            let _ = unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
            if let Some(h) = self.join.take() {
                let _ = h.join();
            }
        }
    }

    impl Drop for ThreadHandle {
        fn drop(&mut self) {
            self.shutdown();
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Headless counterpart: only the join handle matters (no message queue).
    pub struct ThreadHandle {
        pub thread_id: u32,
        join: Option<std::thread::JoinHandle<()>>,
    }

    impl ThreadHandle {
        pub fn new(thread_id: u32, join: std::thread::JoinHandle<()>) -> Self {
            Self {
                thread_id,
                join: Some(join),
            }
        }

        pub fn from_current_thread(join: std::thread::JoinHandle<()>) -> Self {
            static SEQ: AtomicU32 = AtomicU32::new(1);
            Self::new(SEQ.fetch_add(1, Ordering::SeqCst), join)
        }

        pub fn shutdown(&mut self) {
            if let Some(h) = self.join.take() {
                let _ = h.join();
            }
        }
    }

    impl Drop for ThreadHandle {
        fn drop(&mut self) {
            self.shutdown();
        }
    }
}

pub use imp::ThreadHandle;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_is_idempotent() {
        let join = std::thread::spawn(|| {});
        let mut h = ThreadHandle::new(42, join);
        h.shutdown();
        h.shutdown();
    }

    #[cfg(not(windows))]
    #[test]
    fn from_current_thread_gives_distinct_ids() {
        let a = ThreadHandle::from_current_thread(std::thread::spawn(|| {}));
        let b = ThreadHandle::from_current_thread(std::thread::spawn(|| {}));
        assert_ne!(a.thread_id, b.thread_id);
    }
}
