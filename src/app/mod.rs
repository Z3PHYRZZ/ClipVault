//! Application state, message enum, and Iced update/view/subscription.
pub mod message;
pub mod state;
pub mod subscription;
pub mod update;
pub mod view;

pub use message::Message;
pub use state::AppState;
