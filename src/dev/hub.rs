//! Routes messages between the editor connection and the engine connection.

use std::sync::Mutex;

use oxframe_protocol::{ErrorKind, Event, Span};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum Msg {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Default)]
pub struct Hub {
    /// All connected editors (several tabs may be open).
    ui: Mutex<Vec<(u64, UnboundedSender<Msg>)>>,
    next_ui: std::sync::atomic::AtomicU64,
    engine: Mutex<Option<UnboundedSender<Msg>>>,
    /// The editor's last `Hello`, replayed to every new engine process.
    hello: Mutex<Option<String>>,
    /// `Building` or `Error` currently in effect, replayed to editors that
    /// connect later.
    status: Mutex<Option<String>>,
}

impl Hub {
    /// Registers an editor connection; returns its id for [`Hub::remove_ui`].
    pub fn add_ui(&self, tx: UnboundedSender<Msg>) -> u64 {
        tracing::debug!(target: "hub", "editor connected");
        if let Some(s) = self.status.lock().expect("lock").clone() {
            let _ = tx.send(Msg::Text(s));
        }
        let id = self
            .next_ui
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.ui.lock().expect("lock").push((id, tx));
        id
    }

    pub fn remove_ui(&self, id: u64) {
        self.ui.lock().expect("lock").retain(|(i, _)| *i != id);
    }

    pub fn set_engine(&self, tx: Option<UnboundedSender<Msg>>) {
        tracing::debug!(target: "hub", "engine connection: {}", tx.is_some());
        *self.engine.lock().expect("lock") = tx;
    }

    pub fn to_ui(&self, msg: Msg) {
        tracing::debug!(target: "hub", "→ ui {}", describe(&msg));
        self.ui
            .lock()
            .expect("lock")
            .retain(|(_, tx)| tx.send(msg.clone()).is_ok());
    }

    pub fn event(&self, e: &Event) {
        self.to_ui(Msg::Text(serde_json::to_string(e).expect("event json")));
    }

    /// Building/Error status: shown now and to editors connecting later.
    pub fn status(&self, e: Option<&Event>) {
        let text = e.map(|e| serde_json::to_string(e).expect("event json"));
        *self.status.lock().expect("lock") = text.clone();
        if let Some(t) = text {
            self.to_ui(Msg::Text(t));
        }
    }

    pub fn build_error(&self, message: String, spans: Vec<Span>) {
        self.status(Some(&Event::Error {
            kind: ErrorKind::Build,
            message,
            spans,
        }));
    }

    pub fn forward_from_ui(&self, msg: Msg) {
        tracing::debug!(target: "hub", "ui → engine {} (engine connected: {})", describe(&msg), self.engine.lock().expect("lock").is_some());
        if let Msg::Text(t) = &msg
            && t.contains("\"Hello\"")
        {
            *self.hello.lock().expect("lock") = Some(t.clone());
        }
        if let Some(tx) = self.engine.lock().expect("lock").as_ref() {
            let _ = tx.send(msg);
        }
    }

    pub fn hello(&self) -> Option<String> {
        self.hello.lock().expect("lock").clone()
    }
}

fn describe(m: &Msg) -> String {
    match m {
        Msg::Text(t) => t.chars().take(40).collect(),
        Msg::Binary(b) => format!("<{} bytes>", b.len()),
    }
}
