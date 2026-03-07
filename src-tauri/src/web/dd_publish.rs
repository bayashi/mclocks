use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const TEMP_DIR_PREFIX: &str = "/tmpdir-";

#[derive(Default)]
struct TempShareStore {
    hash_to_root: HashMap<String, PathBuf>,
    normalized_root_to_hash: HashMap<String, String>,
}

static TEMP_SHARE_STORE: OnceLock<Mutex<TempShareStore>> = OnceLock::new();

fn share_store() -> &'static Mutex<TempShareStore> {
    TEMP_SHARE_STORE.get_or_init(|| Mutex::new(TempShareStore::default()))
}

fn canonicalize_root(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))
}

fn normalize_root_key(path: &Path) -> String {
    let key = path.to_string_lossy().to_string();
    if cfg!(windows) {
        key.to_lowercase()
    } else {
        key
    }
}

fn create_short_hash(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())[0..8].to_string()
}

pub fn register_temp_root(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let canonical = canonicalize_root(path)?;
    let normalized = normalize_root_key(canonical.as_path());

    let mut store = share_store()
        .lock()
        .map_err(|e| format!("Failed to lock temp share store: {}", e))?;

    if let Some(existing_hash) = store.normalized_root_to_hash.get(&normalized) {
        return Ok(existing_hash.clone());
    }

    let mut hash = create_short_hash(&normalized);
    if let Some(existing_root) = store.hash_to_root.get(&hash) {
        if normalize_root_key(existing_root.as_path()) != normalized {
            let mut suffix = 1u32;
            loop {
                let candidate = format!("{}-{}", hash, suffix);
                if !store.hash_to_root.contains_key(&candidate) {
                    hash = candidate;
                    break;
                }
                suffix += 1;
            }
        }
    }

    store.hash_to_root.insert(hash.clone(), canonical);
    store
        .normalized_root_to_hash
        .insert(normalized, hash.clone());
    Ok(hash)
}

pub fn resolve_temp_share(path: &str) -> Option<(PathBuf, String, String)> {
    if !path.starts_with(TEMP_DIR_PREFIX) {
        return None;
    }

    let remaining = &path[TEMP_DIR_PREFIX.len()..];
    if remaining.is_empty() {
        return None;
    }

    let (hash, suffix_path) = match remaining.find('/') {
        Some(idx) => (&remaining[..idx], &remaining[idx..]),
        None => (remaining, "/"),
    };

    if hash.is_empty() {
        return None;
    }

    let store = share_store().lock().ok()?;
    let root = store.hash_to_root.get(hash)?.clone();
    let relative_path = if suffix_path.is_empty() {
        "/"
    } else {
        suffix_path
    };
    let public_prefix = format!("/tmpdir-{}", hash);
    Some((root, relative_path.to_string(), public_prefix))
}

pub fn build_temp_share_url(port: u16, hash: &str) -> String {
    format!("http://127.0.0.1:{}/tmpdir-{}/", port, hash)
}
