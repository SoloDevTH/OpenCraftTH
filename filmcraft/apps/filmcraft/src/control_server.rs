//! Localhost JSON-lines control server (one request per line, one reply per line). Loopback only.
//!
//! The server only ever reads requests: a line that is not a JSON object with a string `method`
//! (an HTTP request line from a browser's cross-origin `fetch`, a stray `nc`, binary junk) gets
//! one error reply and the connection is closed, so nothing after it is executed. Lines are
//! bounded ([`MAX_LINE`]) and so is the number of open connections ([`MAX_CONNECTIONS`]); junk
//! never reaches the UI thread. See `docs/control-protocol.md`.
//!
//! Every connection must authenticate first: until an `auth` request names the launch's token
//! ([`filmcraft_engine::control_auth`]), any request is refused and the connection closed, so
//! another local program can't drive the app.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use filmcraft_engine::control_auth::{AUTH_METHOD, token_matches};
use filmcraft_ui_egui::ControlRequest;
use serde_json::{Value, json};

/// Longest accepted request line (bytes, without the newline). Requests are small JSON objects;
/// anything longer is refused and the connection closed.
pub const MAX_LINE: usize = 4 * 1024 * 1024;
/// Connections served at once; further ones get an error line and are closed.
pub const MAX_CONNECTIONS: usize = 16;
/// How long a connection may stay unauthenticated: an idle one can't hold a slot forever.
const AUTH_TIMEOUT: Duration = if cfg!(test) { Duration::from_secs(1) } else { Duration::from_secs(10) };

pub fn start(port: u16, token: String, ctx: egui::Context) -> Receiver<ControlRequest> {
    let (tx, rx) = channel::<ControlRequest>();
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("filmcraft: control server failed to bind 127.0.0.1:{port}: {e}");
            return rx;
        }
    };
    eprintln!("filmcraft: control server listening on 127.0.0.1:{port}");
    std::thread::spawn(move || accept_loop(listener, token.into(), tx, ctx));
    rx
}

fn accept_loop(listener: TcpListener, token: Arc<str>, tx: Sender<ControlRequest>, ctx: egui::Context) {
    let open = Arc::new(AtomicUsize::new(0));
    for mut stream in listener.incoming().flatten() {
        if open.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
            open.fetch_sub(1, Ordering::SeqCst);
            let _ = writeln!(stream, "{}", json!({"ok": false, "error": "too many control connections"}));
            continue;
        }
        let tx = tx.clone();
        let ctx = ctx.clone();
        // The slot is given back when the guard drops: after `serve` returns, if it panics, or if
        // the thread can't be started (the closure, and with it the guard, is dropped).
        let slot = Slot(Arc::clone(&open));
        let token = Arc::clone(&token);
        let spawned = std::thread::Builder::new().name("control-conn".into()).spawn(move || {
            let _slot = slot;
            serve(stream, &token, &tx, &ctx);
        });
        if let Err(e) = spawned {
            eprintln!("filmcraft: control server could not start a connection thread: {e}");
        }
    }
}

/// One counted connection; decrements the open-connection count when dropped.
struct Slot(Arc<AtomicUsize>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn serve(stream: TcpStream, token: &str, tx: &Sender<ControlRequest>, ctx: &egui::Context) {
    let Ok(read) = stream.try_clone() else { return };
    // Until `auth` succeeds reads time out; then the client may stay idle (an MCP session). The
    // timeout is set and cleared on the handle that reads: on Windows each handle has its own.
    let _ = read.set_read_timeout(Some(AUTH_TIMEOUT));
    let authenticated = |read: &TcpStream| {
        let _ = read.set_read_timeout(None);
    };
    serve_lines(read, stream, token, authenticated, |method, params| {
        let (req, rrx) = ControlRequest::new(method, params);
        tx.send(req).ok()?;
        ctx.request_repaint();
        Some(rrx.recv_timeout(Duration::from_secs(60)).unwrap_or_else(|_| json!({"ok": false, "error": "timeout"})))
    });
}

/// One parsed line.
enum Line {
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    Blank,
    /// Not a request: reply with this error and close the connection.
    Reject(String),
}

fn parse_line(bytes: &[u8]) -> Line {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Line::Reject("not a control request (invalid UTF-8); closing the connection".into());
    };
    let text = text.trim();
    if text.is_empty() {
        return Line::Blank;
    }
    let msg = match serde_json::from_str::<Value>(text) {
        Ok(v) => v,
        Err(e) => return Line::Reject(format!("bad JSON ({e}); expected one JSON request object per line; closing the connection")),
    };
    let Some(obj) = msg.as_object() else {
        return Line::Reject("not a control request (expected a JSON object); closing the connection".into());
    };
    let Some(method) = obj.get("method").and_then(Value::as_str) else {
        return Line::Reject("not a control request (missing string `method`); closing the connection".into());
    };
    Line::Request { id: obj.get("id").cloned().unwrap_or(Value::Null), method: method.to_string(), params: obj.get("params").cloned().unwrap_or(json!({})) }
}

/// Read one `\n`-terminated line of at most `max` bytes into `buf` (newline stripped).
/// `Ok(false)` at end of stream; `Err` when the line is too long or the read fails.
fn read_bounded_line(reader: &mut impl BufRead, buf: &mut Vec<u8>, max: usize) -> Result<bool, String> {
    buf.clear();
    // `take` bounds the bytes buffered for one line: a peer can't make us allocate more than `max`.
    let limit = u64::try_from(max).unwrap_or(u64::MAX).saturating_add(1);
    let n = reader.take(limit).read_until(b'\n', buf).map_err(|e| match e.kind() {
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => "timed out waiting for authentication; closing the connection".to_string(),
        _ => e.to_string(),
    })?;
    if n == 0 {
        return Ok(false);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
        return Ok(true);
    }
    if buf.len() > max {
        return Err(format!("request line longer than {max} bytes; closing the connection"));
    }
    // Last line without a newline, then end of stream.
    Ok(true)
}

/// The reply to an `auth` request, or to any request before one succeeded (refused: the caller
/// then closes the connection).
fn auth_reply(authenticated: &mut bool, token: &str, method: &str, params: &Value) -> Value {
    if *authenticated {
        return json!({"ok": false, "error": "already authenticated"});
    }
    let supplied = params.get("token").and_then(Value::as_str).unwrap_or("");
    *authenticated = method == AUTH_METHOD && token_matches(token, supplied);
    if *authenticated { json!({"ok": true, "result": {"authenticated": true}}) } else { json!({"ok": false, "error": "authentication required"}) }
}

/// Serve requests from `read`, answering on `out`, until the peer closes, sends something that is
/// not a request, fails to authenticate with `token` first, or `handle` returns `None` (app gone).
/// `handle` gets `(method, params)` and returns the reply object (the request's `id` is added here);
/// `on_authenticated` runs once, when the token is accepted, with the reading half.
fn serve_lines<R: Read, W: Write>(
    read: R,
    mut out: W,
    token: &str,
    mut on_authenticated: impl FnMut(&R),
    mut handle: impl FnMut(String, Value) -> Option<Value>,
) {
    let mut reader = BufReader::new(read);
    let mut buf = Vec::new();
    let mut authenticated = false;
    loop {
        let reject = match read_bounded_line(&mut reader, &mut buf, MAX_LINE) {
            Ok(false) => return,
            Err(e) => e,
            Ok(true) => match parse_line(&buf) {
                Line::Blank => continue,
                Line::Reject(e) => e,
                Line::Request { id, method, params } => {
                    let reply = if authenticated && method != AUTH_METHOD {
                        handle(method, params)
                    } else {
                        let reply = auth_reply(&mut authenticated, token, &method, &params);
                        if authenticated {
                            on_authenticated(reader.get_ref());
                        }
                        Some(reply)
                    };
                    let Some(mut r) = reply else { return };
                    if let Some(o) = r.as_object_mut() {
                        o.insert("id".into(), id);
                    }
                    if writeln!(out, "{r}").and_then(|()| out.flush()).is_err() || !authenticated {
                        return;
                    }
                    continue;
                }
            },
        };
        let _ = writeln!(out, "{}", json!({"ok": false, "error": reject})).and_then(|()| out.flush());
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn auth_line(token: &str) -> String {
        format!("{}\n", json!({"id": "auth", "method": "auth", "params": {"token": token}}))
    }

    /// Serve `input` as is: `(replies, methods handed to the app)`.
    fn run_raw(input: &[u8]) -> (Vec<Value>, Vec<String>) {
        let mut out = Vec::new();
        let mut calls = Vec::new();
        serve_lines(
            input,
            &mut out,
            TOKEN,
            |_| {},
            |m, _p| {
                calls.push(m);
                Some(json!({"ok": true, "result": null}))
            },
        );
        let replies = String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str::<Value>(l).unwrap()).collect();
        (replies, calls)
    }

    /// Serve `input` on an authenticated connection (the auth reply is checked and dropped).
    fn run(input: &[u8]) -> (Vec<Value>, Vec<String>) {
        let mut authed = auth_line(TOKEN).into_bytes();
        authed.extend_from_slice(input);
        let (mut replies, calls) = run_raw(&authed);
        let auth = replies.remove(0);
        assert_eq!((&auth["id"], &auth["ok"]), (&json!("auth"), &json!(true)), "{auth}");
        (replies, calls)
    }

    #[test]
    fn unauthenticated_requests_are_refused_and_the_connection_closed() {
        let wrong = "f".repeat(64);
        for first in [
            "{\"id\":1,\"method\":\"engine.execute\",\"params\":{\"command\":\"file.openDemoProject\"}}\n".to_string(),
            "{\"id\":1,\"method\":\"auth\"}\n".to_string(),
            "{\"id\":1,\"method\":\"auth\",\"params\":{\"token\":\"short\"}}\n".to_string(),
            auth_line(&wrong),
        ] {
            let (r, calls) = run_raw(format!("{first}{}{{\"method\":\"a\"}}\n", auth_line(TOKEN)).as_bytes());
            assert!(calls.is_empty(), "nothing may run before authentication: {calls:?}");
            assert_eq!(r.len(), 1, "the connection closes after the refusal: {r:?}");
            assert_eq!(r[0]["ok"], false);
            assert_eq!(r[0]["error"], "authentication required");
        }
    }

    #[test]
    fn the_right_token_is_accepted_and_a_repeat_auth_is_not_dispatched() {
        let (r, calls) = run(format!("{}{{\"id\":2,\"method\":\"a\"}}\n", auth_line(&TOKEN.to_ascii_uppercase())).as_bytes());
        assert_eq!(calls, ["a"]);
        assert_eq!(r[0]["error"], "already authenticated");
        assert_eq!(r[1]["id"], 2);
        assert_eq!(r[1]["ok"], true);
    }

    #[test]
    fn requests_are_answered_in_order_with_ids() {
        let (r, calls) = run(b"{\"id\":1,\"method\":\"a\"}\n\n{\"id\":2,\"method\":\"b\",\"params\":{}}\r\n{\"method\":\"c\"}");
        assert_eq!(calls, ["a", "b", "c"]);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0]["id"], 1);
        assert_eq!(r[1]["id"], 2);
        assert_eq!(r[2]["id"], Value::Null);
    }

    #[test]
    fn http_request_closes_before_the_body_runs() {
        // A browser's cross-origin `fetch` POST (issue #119): the request line is rejected and the
        // connection closed, so the JSON body is never executed.
        let http = b"POST / HTTP/1.1\r\nHost: 127.0.0.1:9876\r\nOrigin: https://attacker.example\r\nContent-Type: text/plain\r\n\r\n\
            {\"id\":7,\"method\":\"engine.execute\",\"params\":{\"command\":\"file.openDemoProject\",\"params\":{}}}\n";
        let (r, calls) = run(http);
        assert!(calls.is_empty(), "nothing may run: {calls:?}");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0]["ok"], false);
        assert!(r[0]["error"].as_str().unwrap().contains("closing the connection"));
    }

    #[test]
    fn non_request_json_closes_the_connection() {
        for junk in [
            &b"[1,2]\n{\"method\":\"a\"}\n"[..],
            b"{\"id\":1}\n{\"method\":\"a\"}\n",
            b"{\"method\":5}\n{\"method\":\"a\"}\n",
            b"\xff\xfe\n{\"method\":\"a\"}\n",
        ] {
            let (r, calls) = run(junk);
            assert!(calls.is_empty(), "{calls:?}");
            assert_eq!(r.len(), 1);
            assert_eq!(r[0]["ok"], false);
        }
    }

    #[test]
    fn a_request_before_junk_still_runs() {
        let (r, calls) = run(b"{\"method\":\"a\"}\nGET / HTTP/1.1\n{\"method\":\"b\"}\n");
        assert_eq!(calls, ["a"]);
        assert_eq!(r.len(), 2);
        assert_eq!(r[1]["ok"], false);
    }

    #[test]
    fn overlong_line_is_refused() {
        // One byte over the limit, then a valid request: neither runs.
        let mut input = vec![b'x'; MAX_LINE + 1];
        input.extend_from_slice(b"\n{\"method\":\"a\"}\n");
        let (r, calls) = run(&input);
        assert!(calls.is_empty(), "{calls:?}");
        assert_eq!(r.len(), 1);
        assert!(r[0]["error"].as_str().unwrap().contains("longer than"));
    }

    #[test]
    fn line_at_the_limit_is_accepted() {
        let mut buf = Vec::new();
        let line = vec![b'a'; 10];
        let mut input = line.clone();
        input.push(b'\n');
        assert_eq!(read_bounded_line(&mut &input[..], &mut buf, 10), Ok(true));
        assert_eq!(buf, line);
        assert!(read_bounded_line(&mut &[b'a'; 11][..], &mut buf, 10).is_err());
    }

    #[test]
    fn tcp_connection_is_closed_after_junk() {
        // End to end over a real loopback socket (the transport the app uses).
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (s, _) = listener.accept().unwrap();
            let read = s.try_clone().unwrap();
            serve_lines(read, s, TOKEN, |_| {}, |_, _| Some(json!({"ok": true})));
        });
        let mut c = TcpStream::connect(addr).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        c.write_all(auth_line(TOKEN).as_bytes()).unwrap();
        c.write_all(b"{\"id\":7,\"method\":\"x\"}\nhello\n{\"method\":\"y\"}\n").unwrap();
        let mut text = String::new();
        c.read_to_string(&mut text).unwrap(); // returns at EOF: the server closed the connection
        server.join().unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3, "{text}");
        assert!(lines[0].contains("\"authenticated\":true"));
        assert!(lines[1].contains("\"id\":7"));
        assert!(lines[2].contains("closing the connection"));
    }

    #[test]
    fn extra_connections_are_refused() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, _rx) = channel::<ControlRequest>();
        std::thread::spawn(move || accept_loop(listener, TOKEN.into(), tx, egui::Context::default()));
        // Idle connections are accepted in order and held open.
        let held: Vec<TcpStream> = (0..16).map(|_| TcpStream::connect(addr).unwrap()).collect();
        let mut extra = TcpStream::connect(addr).unwrap();
        extra.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut line = String::new();
        BufReader::new(&mut extra).read_line(&mut line).unwrap();
        assert!(line.contains("too many control connections"), "{line}");
        drop(held);
    }

    #[test]
    fn idle_unauthenticated_connections_are_closed_but_authenticated_ones_may_idle() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = channel::<ControlRequest>();
        std::thread::spawn(move || accept_loop(listener, TOKEN.into(), tx, egui::Context::default()));
        // A connection that never authenticates is closed after AUTH_TIMEOUT, freeing its slot.
        let mut idle = TcpStream::connect(addr).unwrap();
        idle.set_read_timeout(Some(AUTH_TIMEOUT * 10)).unwrap();
        let mut text = String::new();
        let _ = idle.read_to_string(&mut text);
        assert!(text.contains("timed out"), "the server must close an idle unauthenticated connection: {text:?}");
        // An authenticated connection may stay idle longer than that.
        let mut authed = TcpStream::connect(addr).unwrap();
        authed.set_read_timeout(Some(AUTH_TIMEOUT * 10)).unwrap();
        authed.write_all(auth_line(TOKEN).as_bytes()).unwrap();
        let mut line = String::new();
        BufReader::new(authed.try_clone().unwrap()).read_line(&mut line).unwrap();
        assert!(line.contains("\"authenticated\":true"), "{line}");
        std::thread::sleep(AUTH_TIMEOUT + Duration::from_millis(500));
        authed.write_all(b"{\"id\":5,\"method\":\"ping\"}\n").unwrap();
        if rx.recv_timeout(Duration::from_secs(5)).is_err() {
            let mut reply = String::new();
            let _ = BufReader::new(&mut authed).read_line(&mut reply);
            panic!("a request after idling must reach the app; the server said {reply:?}");
        }
    }
}
