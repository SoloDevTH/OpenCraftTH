//! Bearer-token authentication for the loopback control channel (`docs/control-protocol.md`).
//!
//! The app (`designcraft --control PORT`) refuses every request on a connection until the first
//! one, `{"method":"auth","params":{"token":"<64 hex>"}}`, names its token; the clients
//! (`designcraft-cli app|script|mcp --connect`) send it after connecting. Same scheme as PhotoCraft.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The first request on every control connection must use this method.
pub const AUTH_METHOD: &str = "auth";
/// Environment equivalents of `--control-token` and `--control-token-file`.
pub const TOKEN_ENV: &str = "DESIGNCRAFT_CONTROL_TOKEN";
pub const TOKEN_FILE_ENV: &str = "DESIGNCRAFT_CONTROL_TOKEN_FILE";

const TOKEN_BYTES: usize = 32;
const TOKEN_HEX_LEN: usize = TOKEN_BYTES * 2;

/// A fresh 256-bit token from the operating system's CSPRNG, as 64 lowercase hex digits.
pub fn generate_token() -> Result<String, String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|e| format!("cannot generate control token: {e}"))?;
    const HEX: &[u8; 16] = b"0123456789abcdef";
    Ok(bytes.iter().flat_map(|b| [HEX[usize::from(b >> 4)] as char, HEX[usize::from(b & 0x0f)] as char]).collect())
}

/// Accept only the fixed-width hexadecimal form [`generate_token`] emits.
pub fn validate_token(token: &str) -> Result<(), String> {
    if token.len() != TOKEN_HEX_LEN || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("control token must contain exactly 64 hexadecimal characters".into());
    }
    Ok(())
}

/// Compare tokens (ignoring hex case) without an early exit on the first differing byte.
pub fn token_matches(expected: &str, supplied: &str) -> bool {
    if expected.len() != TOKEN_HEX_LEN || supplied.len() != TOKEN_HEX_LEN {
        return false;
    }
    expected.bytes().zip(supplied.bytes()).fold(0u8, |diff, (a, b)| diff | (a.to_ascii_lowercase() ^ b.to_ascii_lowercase())) == 0
}

fn read_token_file(path: &Path) -> Result<String, String> {
    let token = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let token = token.trim();
    validate_token(token).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(token.to_ascii_lowercase())
}

/// Create `path` holding `token`; never overwrites (create-new), owner-only on Unix.
fn create_token_file(path: &Path, token: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    writeln!(options.open(path)?, "{token}")
}

/// `--control-token` / `--control-token-file`, falling back to [`TOKEN_ENV`] / [`TOKEN_FILE_ENV`].
/// A token given on the command line is visible to other users in the process list: warn.
pub fn token_inputs(supplied: Option<String>, token_file: Option<PathBuf>) -> (Option<String>, Option<PathBuf>) {
    if supplied.is_some() {
        eprintln!("warning: --control-token is visible to other users in the process list; prefer --control-token-file or {TOKEN_ENV}");
    }
    let supplied = supplied.or_else(|| std::env::var(TOKEN_ENV).ok().filter(|t| !t.is_empty()));
    let token_file = token_file.or_else(|| std::env::var_os(TOKEN_FILE_ENV).filter(|p| !p.is_empty()).map(PathBuf::from));
    (supplied, token_file)
}

/// The app's token: the supplied one, the one in `token_file` (created with a fresh token when
/// missing), or a fresh one for this launch.
pub fn server_token(supplied: Option<&str>, token_file: Option<&Path>) -> Result<String, String> {
    if supplied.is_some() && token_file.is_some() {
        return Err("use either a control token or a control token file, not both".into());
    }
    if let Some(token) = supplied {
        validate_token(token)?;
        return Ok(token.to_ascii_lowercase());
    }
    let Some(path) = token_file else { return generate_token() };
    if path.exists() {
        return read_token_file(path);
    }
    let token = generate_token()?;
    match create_token_file(path, &token) {
        Ok(()) => Ok(token),
        // Another process created it first: use theirs.
        Err(_) if path.exists() => read_token_file(path),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// A client's token. Clients never make one up: without a token or token file this is an error.
pub fn client_token(supplied: Option<&str>, token_file: Option<&Path>) -> Result<String, String> {
    if supplied.is_some() && token_file.is_some() {
        return Err("use either a control token or a control token file, not both".into());
    }
    if let Some(token) = supplied {
        validate_token(token)?;
        return Ok(token.to_ascii_lowercase());
    }
    match token_file {
        Some(path) => read_token_file(path),
        None => Err(format!(
            "the app's control channel needs its token: pass --control-token-file PATH or --control-token HEX (or set {TOKEN_FILE_ENV} / {TOKEN_ENV})"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_valid_and_distinct() {
        let (a, b) = (generate_token().unwrap(), generate_token().unwrap());
        validate_token(&a).unwrap();
        assert_ne!(a, b);
        assert!(token_matches(&a, &a.to_ascii_uppercase()));
        assert!(!token_matches(&a, &b));
        assert!(!token_matches(&a, ""));
        assert!(validate_token("abc").is_err());
        assert!(validate_token(&"g".repeat(64)).is_err());
    }

    #[test]
    fn token_file_is_created_once_and_then_reused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("control.token");
        let first = server_token(None, Some(&path)).unwrap();
        assert_eq!(server_token(None, Some(&path)).unwrap(), first);
        assert_eq!(client_token(None, Some(&path)).unwrap(), first);
        assert!(client_token(None, None).is_err(), "a client never invents a token");
        assert!(server_token(Some(&first), Some(&path)).is_err(), "token and token file are exclusive");
        std::fs::write(&path, "not a token\n").unwrap();
        assert!(server_token(None, Some(&path)).is_err(), "a corrupt token file is an error, not replaced");
    }
}
