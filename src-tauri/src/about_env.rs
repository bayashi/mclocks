//! Short OS / WebView strings for About dialog and clipboard (support diagnostics).
//!
//! Linux webview line: best-effort `dpkg`/`rpm` on common package names, not the loaded
//! `libwebkit2gtk` (Tauri/wry does not expose it). Flatpak/AppImage/snap may disagree.

use tauri_plugin_os::{arch, platform, version};

/// Multi-line text: app version, then OS line, then webview runtime (no field labels).
pub fn format_about_clipboard_text(app_version: &str) -> String {
    format!(
        "mclocks v{}\n{} {} ({})\n{}",
        app_version,
        platform(),
        version(),
        arch(),
        webview_runtime_label()
    )
}

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
