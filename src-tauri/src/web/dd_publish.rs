use super::common::is_supported_web_file;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use urlencoding::{decode, encode};

pub const TEMP_DIR_PREFIX: &str = "/tmpdir-";
pub const TEMP_FILE_PREFIX: &str = "/tmpfile-";

#[derive(Default)]
struct TempShareStore {
    hash_to_root: HashMap<String, PathBuf>,
    normalized_root_to_hash: HashMap<String, String>,
    hash_to_file: HashMap<String, PathBuf>,
    normalized_file_to_hash: HashMap<String, String>,
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

pub fn register_temp_file(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }
    if !is_supported_web_file(path) {
        return Err(format!("Unsupported file type: {}", path.display()));
    }

    let canonical = canonicalize_root(path)?;
    let normalized = normalize_root_key(canonical.as_path());

    let mut store = share_store()
        .lock()
        .map_err(|e| format!("Failed to lock temp share store: {}", e))?;

    if let Some(existing_hash) = store.normalized_file_to_hash.get(&normalized) {
        return Ok(existing_hash.clone());
    }

    let mut hash = create_short_hash(&normalized);
    if let Some(existing_file) = store.hash_to_file.get(&hash) {
        if normalize_root_key(existing_file.as_path()) != normalized {
            let mut suffix = 1u32;
            loop {
                let candidate = format!("{}-{}", hash, suffix);
                if !store.hash_to_file.contains_key(&candidate) {
                    hash = candidate;
                    break;
                }
                suffix += 1;
            }
        }
    }

    store.hash_to_file.insert(hash.clone(), canonical);
    store
        .normalized_file_to_hash
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
    let public_prefix = format!("{}{}", TEMP_DIR_PREFIX, hash);
    Some((root, relative_path.to_string(), public_prefix))
}

pub fn resolve_temp_file(path: &str) -> Option<PathBuf> {
    if !path.starts_with(TEMP_FILE_PREFIX) {
        return None;
    }

    let remaining = &path[TEMP_FILE_PREFIX.len()..];
    let slash_pos = remaining.find('/')?;
    let hash = &remaining[..slash_pos];
    let filename_segment = &remaining[slash_pos + 1..];

    if hash.is_empty() || filename_segment.is_empty() || filename_segment.contains('/') {
        return None;
    }

    let decoded_name = decode(filename_segment).ok()?;
    let store = share_store().lock().ok()?;
    let file_path = store.hash_to_file.get(hash)?.clone();
    let actual_name = file_path.file_name()?.to_string_lossy().to_string();
    if decoded_name != actual_name {
        return None;
    }
    Some(file_path)
}

pub fn build_temp_share_url(port: u16, hash: &str) -> String {
    format!(
        "http://127.0.0.1:{}{}{}/?mode=source",
        port, TEMP_DIR_PREFIX, hash
    )
}

pub fn build_temp_file_url(port: u16, hash: &str, path: &Path) -> Result<String, String> {
    let file_name = path
        .file_name()
        .ok_or("File name not found".to_string())?
        .to_string_lossy()
        .to_string();
    let encoded_name = encode(&file_name);
    Ok(format!(
        "http://127.0.0.1:{}{}{}/{}?mode=source",
        port, TEMP_FILE_PREFIX, hash, encoded_name
    ))
}

#[cfg(test)]
mod tests {
    use super::{build_temp_file_url, build_temp_share_url, register_temp_file};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_register_temp_file_accepts_toml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("bar.toml");
        fs::write(&file_path, "name = \"bar\"\n").expect("Failed to write TOML file");

        let result = register_temp_file(&file_path);

        assert!(
            result.is_ok(),
            "TOML file should be accepted for temp sharing"
        );
    }

    #[test]
    fn test_register_temp_file_accepts_ini_family() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        for ext in ["ini", "config", "cfg"] {
            let file_path = temp_dir.path().join(format!("sample.{}", ext));
            fs::write(&file_path, "name=alice\n").expect("Failed to write INI family file");
            let result = register_temp_file(&file_path);
            assert!(
                result.is_ok(),
                "INI family file should be accepted for temp sharing: .{}",
                ext
            );
        }
    }

    #[test]
    fn test_register_temp_file_accepts_xml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("sample.xml");
        fs::write(&file_path, "<root><name>alice</name></root>").expect("Failed to write XML file");

        let result = register_temp_file(&file_path);

        assert!(
            result.is_ok(),
            "XML file should be accepted for temp sharing"
        );
    }

    #[test]
    fn test_build_temp_share_url_appends_render_mode() {
        let url = build_temp_share_url(3030, "abc12345");
        assert_eq!(url, "http://127.0.0.1:3030/tmpdir-abc12345/?mode=source");
    }

    #[test]
    fn test_build_temp_file_url_appends_render_mode() {
        let path = PathBuf::from("foo.md");
        let url = build_temp_file_url(3030, "abc12345", path.as_path())
            .expect("Failed to build temp file URL");
        assert_eq!(
            url,
            "http://127.0.0.1:3030/tmpfile-abc12345/foo.md?mode=source"
        );
    }
}
