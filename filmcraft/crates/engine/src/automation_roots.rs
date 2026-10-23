//! Launch-time file roots for automation (the `--control` channel and the MCP server), the same
//! policy as PhotoCraft's `AuthorizedWorkspace`.
//!
//! Paths in automation requests are untrusted. [`AutomationRoots::resolve`] accepts a
//! forward-slash relative path beneath a root, or an absolute path whose text lies beneath it, and
//! refuses `..`, `.`, empty components, `\`, drive/stream prefixes and device names before any
//! I/O. It then follows links on the part of the path that exists and refuses it unless the real
//! location is still beneath the root, so a link inside a root cannot point out of it. Either
//! authority may be absent; absent authority fails closed. Interactive use never goes through
//! here.
//!
//! [`AutomationRoots::authorize`] applies this to engine commands: it rewrites the path parameters
//! (`path`, `paths`, `folder`, `dir`, `destination`, `location`, …) and any other absolute-looking string, at any depth, to the resolved paths, and
//! refuses the commands that copy the files a project refers to rather than one the request
//! names.

use std::path::{Component, Path, PathBuf};

use serde_json::Value;

/// Environment equivalents of `--automation-read-root` and `--automation-write-root`.
pub const READ_ROOT_ENV: &str = "FILMCRAFT_AUTOMATION_READ_ROOT";
pub const WRITE_ROOT_ENV: &str = "FILMCRAFT_AUTOMATION_WRITE_ROOT";

const DENIED: &str = "automation filesystem access is not granted";

/// What a request does with a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
    /// Commands whose direction is not known: the path must lie beneath both roots.
    ReadWrite,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Root {
    /// The root as given, made absolute without resolving links: absolute request paths are
    /// compared with this text.
    text: PathBuf,
    /// The root with links resolved: where resolved request paths must stay.
    real: PathBuf,
}

/// Separate read and write roots. The default grants nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AutomationRoots {
    read: Option<Root>,
    write: Option<Root>,
}

impl AutomationRoots {
    /// Open the configured roots; each must be an existing directory.
    pub fn new(read: Option<&Path>, write: Option<&Path>) -> Result<Self, String> {
        Ok(Self { read: open_root(read, "read")?, write: open_root(write, "write")? })
    }

    /// [`Self::new`] from `--automation-read-root` / `--automation-write-root`, falling back to
    /// [`READ_ROOT_ENV`] / [`WRITE_ROOT_ENV`] (an empty variable counts as unset).
    pub fn configured(read: Option<PathBuf>, write: Option<PathBuf>) -> Result<Self, String> {
        let env = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from);
        let read = read.or_else(|| env(READ_ROOT_ENV));
        let write = write.or_else(|| env(WRITE_ROOT_ENV));
        Self::new(read.as_deref(), write.as_deref())
    }

    /// The absolute path an automation request may use for `raw`, or why it may not.
    pub fn resolve(&self, raw: &str, access: Access) -> Result<PathBuf, String> {
        let (first, second) = match access {
            Access::Read => (self.read.as_ref().ok_or_else(|| absent("read"))?, None),
            Access::Write => (self.write.as_ref().ok_or_else(|| absent("write"))?, None),
            Access::ReadWrite => (self.write.as_ref().ok_or_else(|| absent("write"))?, Some(self.read.as_ref().ok_or_else(|| absent("read"))?)),
        };
        let path = confine(first, raw)?;
        if let Some(root) = second {
            confine(root, &path.to_string_lossy())?;
        }
        Ok(path)
    }

    /// `params` of engine command `command` with every path in them resolved, or the reason the
    /// command is refused. A path is the value of a path parameter ([`is_path_key`]), or any other
    /// string that looks absolute (outside free-text fields), at any depth.
    pub fn authorize(&self, command: &str, mut params: Value) -> Result<Value, String> {
        if refuses(command, &params) {
            return Err(format!("automation command `{command}` reads files a document refers to, outside the automation roots, and is disabled"));
        }
        self.confine_tree(command, access(command), &mut params)?;
        Ok(params)
    }

    fn confine_tree(&self, command: &str, access: Access, value: &mut Value) -> Result<(), String> {
        match value {
            Value::Object(map) => {
                for (key, v) in map.iter_mut() {
                    match v {
                        Value::String(raw) if is_path_key(command, key) || (looks_absolute(raw) && !TEXT_KEYS.contains(&key.as_str())) => {
                            *raw = self.resolve_string(raw, access)?
                        }
                        Value::Array(items) if is_path_key(command, key) => {
                            for item in items {
                                match item {
                                    Value::String(raw) => *raw = self.resolve_string(raw, access)?,
                                    other => self.confine_tree(command, access, other)?,
                                }
                            }
                        }
                        other => self.confine_tree(command, access, other)?,
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    match item {
                        Value::String(raw) if looks_absolute(raw) => *raw = self.resolve_string(raw, access)?,
                        other => self.confine_tree(command, access, other)?,
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_string(&self, raw: &str, access: Access) -> Result<String, String> {
        let path = self.resolve(raw, access)?;
        path.to_str().map(str::to_string).ok_or_else(|| path_error(raw, "the resolved path is not valid Unicode"))
    }
}

// ---- FilmCraft's commands ----

/// Commands that copy the media files a project refers to, wherever they are.
fn refuses(command: &str, _params: &Value) -> bool {
    command == "file.projectManager"
}

fn access(command: &str) -> Access {
    let has = |words: &[&str]| words.iter().any(|w| command.contains(w));
    if has(&["export", "Export", "save", "Save", "render", "Render", "proxy", "Proxy", "proxies", "extractAudio", "record", "Record", "narration", "voiceover"])
    {
        Access::Write
    } else if has(&["open", "Open", "import", "Import", "relink", "Relink", "search", "mediaBrowser", "paste", "load", "Load", "Lut", "lut."]) {
        Access::Read
    } else {
        Access::ReadWrite
    }
}

/// Parameters that name files, relative ones included (`path`, `paths`, `folder`, `dir`,
/// `destination`, `location` and keys ending in `Path`, `Folder`, `Dir`, `Location`).
fn is_path_key(_command: &str, key: &str) -> bool {
    matches!(key, "path" | "paths" | "folder" | "dir" | "destination" | "location")
        || ["Path", "Paths", "Folder", "Dir", "Location", "_path", "_dir", "_folder", "_location"].iter().any(|end| key.ends_with(end))
}

// ---- The same in every app ----

/// Free text: not taken for a path because it starts with `/` (a path parameter still is one).
const TEXT_KEYS: &[&str] = &[
    "text",
    "name",
    "title",
    "label",
    "content",
    "query",
    "find",
    "replace",
    "description",
    "comment",
    "notes",
    "message",
    "caption",
    "csv",
    "json",
    "xml",
    "html",
    "base64",
    "bytes",
    "data",
    "script",
];

/// A string another parameter could pass on as a path that leaves the roots: absolute, home-relative
/// or a `file:` URL.
fn looks_absolute(s: &str) -> bool {
    s.starts_with(['/', '\\'])
        || s.starts_with("~/")
        || s.starts_with("~\\")
        || matches!(s.as_bytes(), [drive, b':', b'/' | b'\\', ..] if drive.is_ascii_alphabetic())
        || s.get(..5).is_some_and(|scheme| scheme.eq_ignore_ascii_case("file:"))
}

fn absent(authority: &str) -> String {
    format!("{DENIED}: {authority} authority is absent (start with --automation-{authority}-root DIR)")
}

fn open_root(path: Option<&Path>, authority: &str) -> Result<Option<Root>, String> {
    let Some(path) = path else { return Ok(None) };
    if path.as_os_str().is_empty() {
        return Err(format!("automation {authority} root is empty"));
    }
    let real = std::fs::canonicalize(path).map_err(|e| format!("cannot open automation {authority} root `{}`: {e}", path.display()))?;
    if !real.is_dir() {
        return Err(format!("automation {authority} root `{}` is not a directory", path.display()));
    }
    let text = std::path::absolute(path).map_err(|e| format!("cannot resolve automation {authority} root `{}`: {e}", path.display()))?;
    Ok(Some(Root { text, real: plain(real) }))
}

/// `raw` beneath `root`: checked as text first, then with links resolved.
fn confine(root: &Root, raw: &str) -> Result<PathBuf, String> {
    let relative = if Path::new(raw).is_absolute() || raw.starts_with(['/', '\\']) {
        let rest = strip_root(raw, &root.text)
            .or_else(|| strip_root(raw, &root.real))
            .ok_or_else(|| path_error(raw, "absolute paths must be inside the automation root"))?;
        // The root itself (a folder to export into, say) is allowed.
        if rest.is_empty() { PathBuf::new() } else { relative_path(&rest, raw)? }
    } else {
        relative_path(raw, raw)?
    };
    let joined = root.real.join(&relative);
    // The longest part that exists, links resolved, must still be beneath the root. A link that
    // does not resolve (dangling, or a loop) is refused rather than written through.
    // ponytail: checked, then opened by path; a link swapped in between by another local process
    // escapes. Directory-handle I/O (PhotoCraft's cap-std) closes that if it matters.
    let mut existing = joined.as_path();
    let mut rest = Vec::new();
    while std::fs::symlink_metadata(existing).is_err() {
        let (Some(parent), Some(name)) = (existing.parent(), existing.file_name()) else { break };
        rest.push(name.to_owned());
        existing = parent;
    }
    let real = std::fs::canonicalize(existing).map(plain).map_err(|e| path_error(raw, &format!("cannot resolve its links: {e}")))?;
    if !real.starts_with(&root.real) {
        return Err(path_error(raw, "it leads outside the automation root through a link"));
    }
    Ok(rest.iter().rev().fold(real, |path, name| path.join(name)))
}

/// The components of absolute `raw` after `root`'s, joined with `/`; `None` when `raw` is not
/// beneath `root`. Drive letters compare without case, and a verbatim `\\?\C:\…` counts as `C:\…`.
fn strip_root(raw: &str, root: &Path) -> Option<String> {
    let request = plain(PathBuf::from(raw));
    let mut request = request.components();
    for want in plain(root.to_path_buf()).components() {
        let same = match (request.next(), want) {
            (Some(Component::Prefix(a)), Component::Prefix(b)) => a.as_os_str().eq_ignore_ascii_case(b.as_os_str()),
            (Some(got), want) => got == want,
            (None, _) => false,
        };
        if !same {
            return None;
        }
    }
    let rest: Vec<String> = request.map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    Some(rest.join("/"))
}

/// PhotoCraft's relative-path rules: `raw` is the request as sent, for the error.
fn relative_path(relative: &str, raw: &str) -> Result<PathBuf, String> {
    if relative.is_empty() {
        return Err(path_error(raw, "path is empty or names the root itself"));
    }
    if relative.contains('\\') {
        return Err(path_error(raw, "alternate separators are not allowed; use `/`"));
    }
    if relative.starts_with('/') {
        return Err(path_error(raw, "absolute paths are not allowed"));
    }
    if relative.contains(':') {
        return Err(path_error(raw, "drive, device and stream prefixes are not allowed"));
    }
    for component in relative.split('/') {
        let reason = match component {
            "" => "empty path components are not allowed",
            "." => "`.` path components are not allowed",
            ".." => "parent traversal is not allowed",
            c if c.ends_with(['.', ' ']) => "path components ending in a dot or space are not allowed",
            c if is_windows_device_name(c) => "reserved device names are not allowed",
            _ => continue,
        };
        return Err(path_error(raw, reason));
    }
    Ok(PathBuf::from(relative))
}

fn is_windows_device_name(component: &str) -> bool {
    let stem = component.split('.').next().unwrap_or(component).to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$")
        || stem.strip_prefix("COM").is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
        || stem.strip_prefix("LPT").is_some_and(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
}

/// Windows `canonicalize` returns `\\?\C:\…`; keep the ordinary `C:\…` form other code expects.
fn plain(path: PathBuf) -> PathBuf {
    match path.to_str().and_then(|s| s.strip_prefix(r"\\?\")) {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => path,
    }
}

fn path_error(path: &str, reason: &str) -> String {
    format!("automation path rejected: {reason}: `{path}`")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scratch(name: &str) -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir().join(format!("filmcraft-automation-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let inside = base.join("inside");
        std::fs::create_dir_all(inside.join("sub")).unwrap();
        std::fs::create_dir_all(base.join("outside")).unwrap();
        (base, inside)
    }

    #[test]
    fn paths_outside_the_root_are_refused_and_inside_accepted() {
        let (base, inside) = scratch("resolve");
        let roots = AutomationRoots::new(Some(&inside), Some(&inside)).unwrap();
        let outside = base.join("outside").join("x.png");
        for bad in [
            outside.to_str().unwrap(),
            "../outside/x.png",
            "sub/../../outside/x.png",
            "./x.png",
            "sub//x.png",
            "sub\\x.png",
            "C:x.png",
            "x.png:stream",
            "NUL.txt",
            "",
        ] {
            assert!(roots.resolve(bad, Access::Write).is_err(), "{bad:?}");
        }
        let real = std::fs::canonicalize(&inside).map(plain).unwrap();
        assert_eq!(roots.resolve(inside.to_str().unwrap(), Access::Write).unwrap(), real, "the root itself");
        #[cfg(windows)]
        assert_eq!(roots.resolve(&format!(r"\\?\{}", inside.join("x.png").display()), Access::Write).unwrap(), real.join("x.png"), "verbatim form");
        assert_eq!(roots.resolve("sub/new/x.png", Access::Write).unwrap(), real.join("sub").join("new").join("x.png"));
        let absolute = inside.join("sub").join("x.png");
        assert_eq!(roots.resolve(absolute.to_str().unwrap(), Access::Read).unwrap(), real.join("sub").join("x.png"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn absent_authority_fails_closed() {
        let (base, inside) = scratch("absent");
        assert!(AutomationRoots::default().resolve("x.png", Access::Read).is_err());
        let read_only = AutomationRoots::new(Some(&inside), None).unwrap();
        assert!(read_only.resolve("x.png", Access::Read).is_ok());
        assert!(read_only.resolve("x.png", Access::Write).is_err());
        assert!(read_only.resolve("x.png", Access::ReadWrite).is_err());
        assert!(AutomationRoots::new(Some(&base.join("missing")), None).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn links_out_of_the_root_are_refused() {
        let (base, inside) = scratch("links");
        std::os::unix::fs::symlink(base.join("outside"), inside.join("escape")).unwrap();
        std::os::unix::fs::symlink(base.join("outside").join("gone"), inside.join("dangling")).unwrap();
        std::os::unix::fs::symlink(inside.join("sub"), inside.join("within")).unwrap();
        let roots = AutomationRoots::new(Some(&inside), Some(&inside)).unwrap();
        assert!(roots.resolve("escape/x.png", Access::Write).is_err());
        assert!(roots.resolve("dangling", Access::Write).is_err());
        assert!(roots.resolve("within/x.png", Access::Write).is_ok());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn commands_get_resolved_paths_or_are_refused() {
        let (base, inside) = scratch("commands");
        let roots = AutomationRoots::new(Some(&inside), Some(&inside)).unwrap();
        let p = roots.authorize("file.saveAs", json!({"path": "a.fcproj"})).unwrap();
        assert!(Path::new(p["path"].as_str().unwrap()).is_absolute());
        assert!(roots.authorize("file.saveAs", json!({"path": "../a.fcproj"})).is_err());
        assert!(roots.authorize("file.projectManager", json!({"destination": "copy"})).is_err());
        let p = roots.authorize("file.import", json!({"paths": ["a.mov", "b.wav"]})).unwrap();
        assert!(p["paths"].as_array().unwrap().iter().all(|v| Path::new(v.as_str().unwrap()).is_absolute()));
        assert!(roots.authorize("file.import", json!({"paths": ["a.mov", "../b.wav"]})).is_err());
        assert!(roots.authorize("settings.set", json!({"scratch": {"capturedLocation": "../x"}})).is_err());
        assert_eq!(roots.authorize("sequence.addEdit", json!({"seconds": 3})).unwrap(), json!({"seconds": 3}));
        assert_eq!(roots.authorize("timecode.set", json!({"timecode": "01:00:00:00"})).unwrap(), json!({"timecode": "01:00:00:00"}));
        // Absolute-looking strings under any key, at any depth, are paths too; free text is not.
        assert!(roots.authorize("prefs.set", json!({"values": {"folder": "/etc"}})).is_err());
        assert!(roots.authorize("prefs.set", json!({"list": ["C:\\Windows"]})).is_err());
        assert!(roots.authorize("prefs.set", json!({"url": "file:///etc/passwd"})).is_err());
        assert_eq!(roots.authorize("text.insert", json!({"text": "/etc"})).unwrap(), json!({"text": "/etc"}));
        let nested = inside.join("sub").to_string_lossy().into_owned();
        assert!(roots.authorize("prefs.set", json!({"values": {"folder": nested}})).is_ok());
        assert!(AutomationRoots::default().authorize("file.save", json!({"path": "a.fcproj"})).is_err());
        assert!(AutomationRoots::default().authorize("file.save", json!({})).is_ok(), "saving back to the document's own file");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_confined_session_writes_only_inside_the_root() {
        let (base, inside) = scratch("session");
        let roots = AutomationRoots::new(Some(&inside), Some(&inside)).unwrap();
        let mut s = crate::Session { automation_roots: Some(roots), ..Default::default() };
        let outside = base.join("outside").join("x.fcproj");
        assert!(s.execute("file.saveAs", json!({"path": outside.to_string_lossy()})).is_err());
        assert!(s.execute("file.saveAs", json!({"path": "../outside/x.fcproj"})).is_err());
        assert!(!outside.exists(), "nothing is written outside the root");
        s.execute("file.saveAs", json!({"path": "x.fcproj"})).unwrap();
        assert!(inside.join("x.fcproj").is_file());
        // Interactive use is not confined.
        s.automation_roots = None;
        s.execute("file.saveAs", json!({"path": outside.to_string_lossy()})).unwrap();
        assert!(outside.is_file());
        let _ = std::fs::remove_dir_all(&base);
    }
}
