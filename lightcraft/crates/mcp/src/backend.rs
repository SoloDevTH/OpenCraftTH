//! Where MCP tool calls end up: a control-channel method call.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde_json::{Value, json};

/// Something that answers control-channel methods (`engine.execute`, `engine.commands`,
/// `ui.render`, `ui.screenshot`, `ui.key`, …). See `docs/control-protocol.md`.
pub trait Backend {
    /// Call one method. `Ok` carries the `result`, `Err` the error message.
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String>;
    /// True when a real UI is attached (`ui.inspect`, `ui.clickWidget`, `ui.pointer` … work).
    fn has_ui(&self) -> bool;
    /// Short human description ("headless", "remote 127.0.0.1:7980").
    fn describe(&self) -> String;
    /// Progress/cancellation for the next headless batch export. Other backends ignore it.
    fn set_progress(&mut self, _hook: Option<ProgressHook>) {}
}

/// `(completed photos, total photos, next filename)`; false stops before the next photo.
pub type ProgressHook = Box<dyn FnMut(usize, usize, &str) -> bool + Send>;

/// A running LightCraft app, reached through its loopback control port.
pub struct Remote {
    addr: String,
    /// The app's control token, sent as each connection's first request (`auth`).
    token: String,
    conn: Option<(BufReader<TcpStream>, TcpStream)>,
    next_id: u64,
}

impl Remote {
    /// Connect to `addr` (e.g. `127.0.0.1:7980`) and authenticate with the app's control `token`
    /// (`lightcraft_engine::control_auth::client_token`), failing fast when nothing is listening or
    /// the token is refused.
    pub fn connect(addr: &str, token: &str) -> std::io::Result<Self> {
        let mut r = Self::lazy(addr, token);
        r.reconnect()?;
        Ok(r)
    }

    /// A remote that connects on first use (the app may start after the MCP server).
    pub fn lazy(addr: &str, token: &str) -> Self {
        Self { addr: addr.to_string(), token: token.to_string(), conn: None, next_id: 1 }
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    fn reconnect(&mut self) -> std::io::Result<()> {
        self.conn = None;
        let mut last = std::io::Error::new(std::io::ErrorKind::NotFound, format!("cannot resolve {}", self.addr));
        for sa in self.addr.to_socket_addrs()? {
            match TcpStream::connect_timeout(&sa, Duration::from_millis(800)) {
                Ok(s) => {
                    s.set_nodelay(true).ok();
                    // The app answers within 60 s (its own timeout); leave headroom.
                    s.set_read_timeout(Some(Duration::from_secs(90))).ok();
                    let read = s.try_clone()?;
                    self.conn = Some((BufReader::new(read), s));
                    let auth = json!({"id": "auth", "method": "auth", "params": {"token": self.token}}).to_string();
                    let reply = self.roundtrip(&auth).and_then(|r| {
                        let v: Value = serde_json::from_str(r.trim()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                        if v.get("ok").and_then(Value::as_bool) == Some(true) {
                            return Ok(());
                        }
                        let e = v.get("error").and_then(Value::as_str).unwrap_or("refused");
                        Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            format!("control authentication failed: {e} (wrong control token?)"),
                        ))
                    });
                    if reply.is_err() {
                        self.conn = None;
                    }
                    return reply;
                }
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    fn roundtrip(&mut self, line: &str) -> std::io::Result<String> {
        if self.conn.is_none() {
            self.reconnect()?;
        }
        let Some((reader, writer)) = self.conn.as_mut() else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "not connected"));
        };
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        let mut reply = String::new();
        if reader.read_line(&mut reply)? == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "control channel closed"));
        }
        Ok(reply)
    }
}

impl Backend for Remote {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let line = json!({"id": id, "method": method, "params": params}).to_string();
        // One retry with a fresh connection (the app may have restarted).
        let reply = match self.roundtrip(&line) {
            Ok(r) => r,
            // A refused token won't be accepted on a retry either.
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Err(format!("LightCraft app at {}: {e}", self.addr)),
            Err(_) => {
                self.conn = None;
                self.roundtrip(&line).map_err(|e| {
                    self.conn = None;
                    format!("LightCraft app at {} is not reachable: {e} (start it with `lightcraft --control PORT`)", self.addr)
                })?
            }
        };
        let v: Value = serde_json::from_str(reply.trim()).map_err(|e| format!("bad reply from app: {e}"))?;
        if v.get("ok").and_then(Value::as_bool) == Some(true) {
            Ok(v.get("result").cloned().unwrap_or(Value::Null))
        } else {
            Err(v.get("error").and_then(Value::as_str).unwrap_or("unknown error").to_string())
        }
    }

    fn has_ui(&self) -> bool {
        true
    }

    fn describe(&self) -> String {
        format!("remote {}", self.addr)
    }
}
