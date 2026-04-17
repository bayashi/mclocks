//! CLI helper: forwards to a running mclocks instance via `--mc-preview` (single-instance channel).
use std::path::{Path, PathBuf};
use std::process::Command;

fn print_usage() {
    eprintln!("usage: mc-preview FILE_PATH|DIRECTORY_PATH");
    eprintln!("Requires mclocks to be running. Opens the path in the built-in web viewer.");
    eprintln!("The mclocks executable must sit next to this program (same directory).");
}

fn mclocks_executable_beside_self() -> Result<PathBuf, String> {
    if let Ok(from_env) = std::env::var("MCLOCKS_EXE") {
        let p = PathBuf::from(from_env);
        if p.exists() {
            return Ok(p);
        }
        return Err(format!(
            "MCLOCKS_EXE is set but file does not exist: {}",
            p.display()
        ));
    }
    let self_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = self_exe
        .parent()
        .ok_or_else(|| "current_exe has no parent directory".to_string())?;
    let name = if cfg!(windows) {
        "mclocks.exe"
    } else {
        "mclocks"
    };
    let candidate = dir.join(name);
    if candidate.exists() {
        return Ok(candidate);
    }
    Err(format!(
        "mclocks not found next to mc-preview (expected {}); set MCLOCKS_EXE to override",
        candidate.display()
    ))
}

fn main() -> Result<(), String> {
    let mut argv = std::env::args_os().skip(1);
    let Some(path_os) = argv.next() else {
        print_usage();
        std::process::exit(2);
    };
    if argv.next().is_some() {
        print_usage();
        std::process::exit(2);
    }
    let path = Path::new(&path_os);
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mclocks = mclocks_executable_beside_self()?;
    let status = Command::new(&mclocks)
        .arg("--mc-preview")
        .arg(&abs)
        .status()
        .map_err(|e| format!("failed to spawn mclocks: {}", e))?;
    if !status.success() {
        return Err(format!("mclocks exited with {}", status));
    }
    Ok(())
}
