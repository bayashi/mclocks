//! WebSocket server for live-reloading rendered Markdown in the external browser.
use futures_util::{SinkExt, StreamExt};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_hdr_async;
use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::http::StatusCode;
use tungstenite::Message;
use uuid::Uuid;

const LIVE_PATH_PREFIX: &str = "/live/";

fn normalize_path_key(path: &Path) -> String {
    let key = path.to_string_lossy().to_string();
    if cfg!(windows) {
        key.to_lowercase()
    } else {
        key
    }
}

fn event_targets_path(event: &Event, target: &Path) -> bool {
    let want = normalize_path_key(target);
    event.paths.iter().any(|p| normalize_path_key(p) == want)
}

fn spawn_path_watcher(watch_path: PathBuf, tx: broadcast::Sender<()>) {
    thread::spawn(move || {
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        let mut watcher = match RecommendedWatcher::new(notify_tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("md live reload: watcher create failed: {}", e);
                return;
            }
        };
        if watcher
            .watch(&watch_path, RecursiveMode::NonRecursive)
            .is_err()
        {
            if let Some(parent) = watch_path.parent() {
                if watcher
                    .watch(parent, RecursiveMode::NonRecursive)
                    .is_err()
                {
                    eprintln!(
                        "md live reload: failed to watch {} or parent",
                        watch_path.display()
                    );
                    return;
                }
            } else {
                eprintln!(
                    "md live reload: failed to watch {}",
                    watch_path.display()
                );
                return;
            }
        }
        while let Ok(res) = notify_rx.recv() {
            match res {
                Ok(ev) => {
                    if !event_targets_path(&ev, &watch_path) {
                        continue;
                    }
                    let _ = tx.send(());
                }
                Err(e) => {
                    eprintln!("md live reload: notify error: {}", e);
                }
            }
        }
    });
}

struct Hub {
    token_to_path: HashMap<String, PathBuf>,
    path_to_tx: HashMap<String, broadcast::Sender<()>>,
}

impl Hub {
    fn new() -> Self {
        Self {
            token_to_path: HashMap::new(),
            path_to_tx: HashMap::new(),
        }
    }

    fn register_session(&mut self, path: PathBuf) -> String {
        let token = Uuid::new_v4().to_string();
        let key = normalize_path_key(&path);
        self.token_to_path.insert(token.clone(), path.clone());
        if !self.path_to_tx.contains_key(&key) {
            let (tx, _rx) = broadcast::channel::<()>(64);
            self.path_to_tx.insert(key.clone(), tx.clone());
            spawn_path_watcher(path, tx);
        }
        token
    }

    fn token_valid(&self, token: &str) -> bool {
        self.token_to_path.contains_key(token)
    }

    fn subscribe(&self, token: &str) -> Option<broadcast::Receiver<()>> {
        let path = self.token_to_path.get(token)?;
        let key = normalize_path_key(path);
        self.path_to_tx.get(&key).map(|t| t.subscribe())
    }
}

fn hub() -> &'static Mutex<Hub> {
    static HUB: OnceLock<Mutex<Hub>> = OnceLock::new();
    HUB.get_or_init(|| Mutex::new(Hub::new()))
}

static WS_PORT: OnceLock<u16> = OnceLock::new();

/// Starts the markdown live-reload WebSocket listener. Call at most once.
/// Blocks until the listener has bound or binding failed.
/// Returns `true` when the socket is listening and [`markdown_live_reload_ws_port`] is set.
pub fn start_markdown_live_reload_server(port: u16) -> bool {
    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(0);
    thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("md live reload: tokio runtime failed: {}", e);
                return;
            }
        };
        rt.block_on(run_ws_server(port, ready_tx));
    });
    match ready_rx.recv() {
        Ok(()) => {
            if WS_PORT.set(port).is_err() {
                eprintln!("md live reload: port already registered");
                return false;
            }
            true
        }
        Err(_) => {
            eprintln!("md live reload: server thread exited before bind");
            false
        }
    }
}

/// Returns the bound port if the server was started.
pub fn markdown_live_reload_ws_port() -> Option<u16> {
    WS_PORT.get().copied()
}

/// Registers a file for live reload and returns a session token for the WebSocket path.
pub fn register_markdown_live_reload_session(file_path: &Path) -> Option<String> {
    if markdown_live_reload_ws_port().is_none() {
        return None;
    }
    let mut g = hub().lock().ok()?;
    Some(g.register_session(file_path.to_path_buf()))
}

fn forbidden_response() -> ErrorResponse {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(Some("Forbidden".to_string()))
        .expect("forbidden response")
}

fn parse_live_token(path: &str) -> Option<&str> {
    path.strip_prefix(LIVE_PATH_PREFIX).map(|s| s.trim_end_matches('/'))
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let mut token_out: Option<String> = None;
    let ws_result = accept_hdr_async(stream, |req: &Request, response: Response| {
        let path = req.uri().path();
        let token = match parse_live_token(path) {
            Some(t) if !t.is_empty() && !t.contains('/') => t.to_string(),
            _ => return Err(forbidden_response()),
        };
        let ok = hub()
            .lock()
            .ok()
            .is_some_and(|h| h.token_valid(&token));
        if !ok {
            return Err(forbidden_response());
        }
        token_out = Some(token);
        Ok(response)
    })
    .await;
    let mut ws = match ws_result {
        Ok(ws) => ws,
        Err(_) => return,
    };
    let token = match token_out {
        Some(t) => t,
        None => return,
    };
    let mut rx = {
        let hub = match hub().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        match hub.subscribe(&token) {
            Some(r) => r,
            None => return,
        }
    };
    loop {
        tokio::select! {
            biased;
            recv = rx.recv() => {
                match recv {
                    Ok(()) => {
                        if ws.send(Message::Text("reload".into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            ws_msg = ws.next() => {
                match ws_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(p))) => {
                        let _ = ws.send(Message::Pong(p)).await;
                    }
                    Some(Err(_)) => break,
                    Some(Ok(_)) => {}
                }
            }
        }
    }
    let _ = ws.close(None).await;
}

async fn run_ws_server(port: u16, ready_tx: std::sync::mpsc::SyncSender<()>) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("md live reload: bind {} failed: {}", addr, e);
            return;
        }
    };
    if ready_tx.send(()).is_err() {
        return;
    }
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                eprintln!("md live reload: accept failed: {}", e);
            }
        }
    }
}
