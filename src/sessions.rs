//! Active player session registry.
//!
//! When a browser opens /events/:session, it registers here.
//! The control UI polls /sessions to see what's playing.
//! POST /command/:session pushes a command over SSE to that browser.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::broadcast;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub media_id: Option<String>,
    pub media_title: Option<String>,
    pub client_ip: String,
    pub last_seen: u64, // unix timestamp
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerCommand {
    pub cmd: String,       // play, pause, toggle, seek, next, prev, stop
    pub value: Option<f64>, // seek offset seconds
}

#[derive(Clone)]
pub struct Session {
    pub info: SessionInfo,
    pub tx: broadcast::Sender<PlayerCommand>,
}

#[derive(Clone, Default)]
pub struct SessionRegistry(Arc<Mutex<HashMap<String, Session>>>);

impl SessionRegistry {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn register(&self, session_id: String, client_ip: String) -> broadcast::Receiver<PlayerCommand> {
        let (tx, rx) = broadcast::channel(32);
        let info = SessionInfo {
            session_id: session_id.clone(),
            media_id: None,
            media_title: None,
            client_ip,
            last_seen: now(),
        };
        self.0.lock().unwrap().insert(session_id, Session { info, tx });
        rx
    }

    pub fn update_media(&self, session_id: &str, media_id: &str, media_title: &str) {
        if let Some(s) = self.0.lock().unwrap().get_mut(session_id) {
            s.info.media_id = Some(media_id.to_string());
            s.info.media_title = Some(media_title.to_string());
            s.info.last_seen = now();
        }
    }

    pub fn heartbeat(&self, session_id: &str) {
        if let Some(s) = self.0.lock().unwrap().get_mut(session_id) {
            s.info.last_seen = now();
        }
    }

    pub fn send_command(&self, session_id: &str, cmd: PlayerCommand) -> bool {
        if let Some(s) = self.0.lock().unwrap().get(session_id) {
            s.tx.send(cmd).is_ok()
        } else {
            false
        }
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        let cutoff = now().saturating_sub(30); // 30s timeout
        self.0.lock().unwrap()
            .values()
            .filter(|s| s.info.last_seen >= cutoff)
            .map(|s| s.info.clone())
            .collect()
    }

    pub fn remove(&self, session_id: &str) {
        self.0.lock().unwrap().remove(session_id);
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
