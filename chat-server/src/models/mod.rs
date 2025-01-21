mod chat;
mod file;
mod message;
mod user;
mod workspace;

pub use chat::ParamChat;
pub use message::{CreateMessage, DeleteMessage, ListMessage};
use serde::{Deserialize, Serialize};
pub use user::{CreateUser, SigninUser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String, // extract ext from filename or use mime type
    pub hash: String,
}
