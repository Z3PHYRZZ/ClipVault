//! GUI subscription: forward all worker events into the message loop,
//! plus a slow retention tick. No tokio/smol runtime feature required —
//! the tick is produced by a dedicated sleeping thread (idle = zero CPU).
use crate::app::{AppState, Message};
use crate::workers::WorkerEvent;
use iced::Subscription;
use iced::futures::SinkExt;

pub fn subscription(state: &AppState) -> Subscription<Message> {
    let rx: flume::Receiver<WorkerEvent> = state.events_rx.clone();
    let worker = Subscription::run_with_id(
        "clipvault-workers",
        iced::stream::channel(256, move |mut output| async move {
            while let Ok(ev) = rx.recv_async().await {
                if output.send(Message::Worker(ev)).await.is_err() {
                    break;
                }
            }
        }),
    );

    // Slow periodic retention tick (default 10 minutes). A sleeping thread,
    // not a busy loop, keeps idle CPU at zero.
    let retention = Subscription::run_with_id(
        "clipvault-retention",
        iced::stream::channel(1, move |mut output| async move {
            loop {
                tokio_free_sleep(std::time::Duration::from_secs(600)).await;
                if output.send(Message::RetentionTick).await.is_err() {
                    break;
                }
            }
        }),
    );

    Subscription::batch(vec![worker, retention])
}

/// `Sleep` without a runtime: blocks a small dedicated thread and resolves a
/// oneshot when elapsed. Runs inside iced's futures executor via `spawn`-like
/// behavior of stream::channel's future. We use a plain thread + flume oneshot
/// so no async runtime is needed.
async fn tokio_free_sleep(d: std::time::Duration) {
    let (tx, rx) = flume::bounded::<()>(1);
    std::thread::spawn(move || {
        std::thread::sleep(d);
        let _ = tx.send(());
    });
    let _ = rx.recv_async().await;
}
