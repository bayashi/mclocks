use std::path::Path;
use std::process::Command;

fn main() {
    tauri_build::build();

    if let Ok(t) = std::env::var("TARGET") {
        println!("cargo:rustc-env=MCLOCKS_BUILD_TARGET={t}");
    }

    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let repo_root = Path::new(&manifest).join("..");

    let git_short = run_git(&repo_root, &["rev-parse", "--short", "HEAD"]);
    let dirty = git_dirty(&repo_root);
    let git_label = match &git_short {
        Some(s) if dirty => format!("{s}-dirty"),
        Some(s) => s.clone(),
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=MCLOCKS_GIT={git_label}");

    let built_utc = build_time_utc_rfc3339();
    println!("cargo:rustc-env=MCLOCKS_BUILD_TIME_UTC={built_utc}");

    let git_head = repo_root.join(".git").join("HEAD");
    if git_head.is_file() {
        println!("cargo:rerun-if-changed={}", git_head.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}

fn run_git(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn git_dirty(repo: &Path) -> bool {
    let Ok(out) = Command::new("git")
        .current_dir(repo)
        .args(["status", "--porcelain"])
        .output()
    else {
        return false;
    };
    out.status.success() && !out.stdout.is_empty()
}

fn build_time_utc_rfc3339() -> String {
    if let Ok(s) = std::env::var("SOURCE_DATE_EPOCH") {
        if let Ok(secs) = s.parse::<i64>() {
            if let Some(dt) = chrono::DateTime::from_timestamp(secs, 0) {
                return dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
            }
        }
    }
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
