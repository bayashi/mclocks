//! Short OS / WebView strings for About dialog and clipboard (support diagnostics).
//!
//! Build/runtime lines: packaged vs `tauri dev`, Rust debug/release profile (cargo),
//! compile target triple (`TARGET`), short git revision from `build.rs`, and UTC build time
//! (`SOURCE_DATE_EPOCH` when set for reproducible builds, else wall clock at compile).
//!
//! Linux webview line: best-effort `dpkg`/`rpm` on common package names, not the loaded
//! `libwebkit2gtk` (Tauri/wry does not expose it). Flatpak/AppImage/snap may disagree.
//!
//! The tray "About" action exists only on Windows and macOS (`tray.rs`), so this module
//! is compiled only for those targets to avoid `dead_code` on Linux CI.

#[cfg(any(target_os = "windows", target_os = "macos"))]
use tauri_plugin_os::{arch, platform, version};

/// Multi-line text: app version, then OS line, then webview runtime (no field labels).
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn format_about_clipboard_text(app_version: &str) -> String {
    let mut out = format!(
        "mclocks v{}\n{} {} ({})\n{}\n{}\n{}\nBuild target: {}",
        app_version,
        platform(),
        version(),
        arch(),
        webview_runtime_label(),
        runtime_kind_label(),
        rust_profile_label(),
        build_target_triple(),
    );
    if let Some(rev) = git_revision_known() {
        out.push_str("\nGit: ");
        out.push_str(rev);
    }
    out.push_str("\nBuilt (UTC): ");
    out.push_str(build_time_utc_label());
    out
}

/// Compile-time target triple (from `build.rs` + Cargo `TARGET`).
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn build_target_triple() -> &'static str {
    option_env!("MCLOCKS_BUILD_TARGET").unwrap_or("(unknown)")
}

/// `None` when git was unavailable at build time (`unknown` or unset).
#[cfg(any(target_os = "windows", target_os = "macos"))]
fn git_revision_known() -> Option<&'static str> {
    match option_env!("MCLOCKS_GIT") {
        None | Some("unknown") | Some("") => None,
        Some(g) => Some(g),
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn build_time_utc_label() -> &'static str {
    option_env!("MCLOCKS_BUILD_TIME_UTC").unwrap_or("unknown")
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn runtime_kind_label() -> &'static str {
    if tauri::is_dev() {
        "Runtime: development (tauri dev)"
    } else {
        "Runtime: packaged"
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn rust_profile_label() -> &'static str {
    if cfg!(debug_assertions) {
        "Rust profile: debug"
    } else {
        "Rust profile: release"
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn webview_runtime_label() -> String {
    #[cfg(windows)]
    {
        webview2_version_from_registry().unwrap_or_else(|| "WebView2 (version unknown)".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        "WKWebView (system WebKit)".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        webkitgtk_version_hint()
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        "(unknown)".to_string()
    }
}

#[cfg(windows)]
fn webview2_version_from_registry() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    const HKCU: &str =
        r"Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    const HKLM_WOW: &str =
        r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
    const HKLM: &str =
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

    for (hive, path) in [
        (HKEY_CURRENT_USER, HKCU),
        (HKEY_LOCAL_MACHINE, HKLM_WOW),
        (HKEY_LOCAL_MACHINE, HKLM),
    ] {
        if let Ok(key) = RegKey::predef(hive).open_subkey(path) {
            if let Ok(pv) = key.get_value::<String, _>("pv") {
                let pv = pv.trim();
                if !pv.is_empty() && pv != "0.0.0.0" {
                    return Some(format!("WebView2 {pv}"));
                }
            }
        }
    }
    None
}

/// Distro package version guess; does not read the process-linked WebKit `.so`.
#[cfg(target_os = "linux")]
fn webkitgtk_version_hint() -> String {
    use std::process::Command;

    let deb_pkgs = [
        "libwebkit2gtk-4.1-0",
        "libwebkit2gtk-4.0-37",
        "libwebkit2gtk-4.0-0",
    ];
    for pkg in deb_pkgs {
        if let Ok(out) = Command::new("dpkg-query")
            .args(["-f", "${Version}", "-W", pkg])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return format!("WebKitGTK {s}");
                }
            }
        }
    }

    if let Ok(out) = Command::new("rpm")
        .args(["-q", "--qf", "%{VERSION}", "webkit2gtk4.1"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() && s != "not installed" {
                return format!("WebKitGTK {s}");
            }
        }
    }

    "WebKitGTK (system)".to_string()
}
