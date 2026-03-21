use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Runtime};
#[cfg(target_os = "windows")]
use tauri_plugin_clipboard_manager::ClipboardExt;
#[cfg(target_os = "windows")]
use tauri_plugin_dialog::MessageDialogResult;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

const MENU_ID_TRAY_TOGGLE_MAIN: &str = "menu.tray.toggle_main";
const MENU_ID_RESET_TEMP_DND_SESSION: &str = "menu.web.reset_temp_dnd_session";
const MENU_ID_TRAY_QUIT: &str = "menu.tray.quit";
#[cfg(target_os = "windows")]
const MENU_ID_TRAY_ABOUT: &str = "menu.tray.about";
const TRAY_LABEL_SHOW_MAIN: &str = "Show mclocks";
const TRAY_LABEL_HIDE_MAIN: &str = "Hide mclocks";
#[cfg(target_os = "windows")]
const TRAY_LABEL_QUIT: &str = "Exit";
#[cfg(target_os = "macos")]
const TRAY_LABEL_QUIT: &str = "Quit";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const TRAY_LABEL_QUIT: &str = "Quit";

fn has_visible_webview_window<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.webview_windows()
        .values()
        .any(|window| window.is_visible().unwrap_or(false))
}

fn hide_all_webview_windows<R: Runtime>(app: &AppHandle<R>) {
    for window in app.webview_windows().values() {
        let _ = window.hide();
    }
}

fn show_all_webview_windows<R: Runtime>(app: &AppHandle<R>, main_window_name: &str) {
    for window in app.webview_windows().values() {
        let _ = window.show();
    }
    if let Some(main_window) = app.get_webview_window(main_window_name) {
        let _ = main_window.set_focus();
    }
}

pub fn setup_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    window_name: &str,
    reset_temp_web_session: fn() -> Result<String, String>,
) -> tauri::Result<()> {
    let toggle_main_item = MenuItem::with_id(
        app,
        MENU_ID_TRAY_TOGGLE_MAIN,
        TRAY_LABEL_HIDE_MAIN,
        true,
        None::<&str>,
    )?;
    let reset_temp_session_item = MenuItem::with_id(
        app,
        MENU_ID_RESET_TEMP_DND_SESSION,
        "Reset Web D&&D URL",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, MENU_ID_TRAY_QUIT, TRAY_LABEL_QUIT, true, None::<&str>)?;
    #[cfg(target_os = "windows")]
    let about_item =
        MenuItem::with_id(app, MENU_ID_TRAY_ABOUT, "About mclocks", true, None::<&str>)?;
    #[cfg(target_os = "windows")]
    let tray_menu = Menu::with_items(
        app,
        &[
            &toggle_main_item,
            &reset_temp_session_item,
            &about_item,
            &quit_item,
        ],
    )?;
    #[cfg(not(target_os = "windows"))]
    let tray_menu = Menu::with_items(
        app,
        &[&toggle_main_item, &reset_temp_session_item, &quit_item],
    )?;
    let mut tray_builder = TrayIconBuilder::with_id("main").menu(&tray_menu);
    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }
    let toggle_main_item_for_event = toggle_main_item.clone();
    if has_visible_webview_window(app) {
        let _ = toggle_main_item.set_text(TRAY_LABEL_HIDE_MAIN);
    } else {
        let _ = toggle_main_item.set_text(TRAY_LABEL_SHOW_MAIN);
    }
    let main_window_name = window_name.to_string();
    #[cfg(target_os = "windows")]
    let about_version_text = format!("v{}", app.package_info().version);
    #[cfg(target_os = "windows")]
    let about_dialog_message = format!("mclocks\n{}", about_version_text);
    tray_builder
        .on_menu_event(move |app, event| {
            let menu_id = event.id().as_ref();
            #[cfg(target_os = "windows")]
            if menu_id == MENU_ID_TRAY_ABOUT {
                let app_handle = app.clone();
                let version_to_copy = about_version_text.clone();
                let message = about_dialog_message.clone();
                app_handle
                    .dialog()
                    .message(message)
                    .title("About")
                    .kind(MessageDialogKind::Info)
                    .buttons(MessageDialogButtons::OkCancelCustom(
                        "OK".to_string(),
                        "Copy version".to_string(),
                    ))
                    .show_with_result(move |res| {
                        if matches!(
                            res,
                            MessageDialogResult::Custom(ref s) if s == "Copy version"
                        ) {
                            let _ = app_handle.clipboard().write_text(&version_to_copy);
                        }
                    });
                return;
            }
            if menu_id == MENU_ID_TRAY_TOGGLE_MAIN {
                if has_visible_webview_window(app) {
                    hide_all_webview_windows(app);
                    let _ = toggle_main_item_for_event.set_text(TRAY_LABEL_SHOW_MAIN);
                } else {
                    show_all_webview_windows(app, &main_window_name);
                    let _ = toggle_main_item_for_event.set_text(TRAY_LABEL_HIDE_MAIN);
                }
                return;
            }
            if menu_id == MENU_ID_TRAY_QUIT {
                app.exit(0);
                return;
            }
            if menu_id != MENU_ID_RESET_TEMP_DND_SESSION {
                return;
            }
            let app_handle = app.clone();
            app.dialog()
                .message(
                    "Reset temporary D&D web session?\nAll temporary URLs opened via D&D will become unavailable.",
                )
                .title("Confirm")
                .kind(MessageDialogKind::Warning)
                .buttons(MessageDialogButtons::OkCancelCustom(
                    "Reset".to_string(),
                    "Cancel".to_string(),
                ))
                .show(move |confirmed| {
                    if !confirmed {
                        return;
                    }
                    match reset_temp_web_session() {
                        Ok(message) => {
                            app_handle
                                .dialog()
                                .message(&message)
                                .title("Web Server")
                                .kind(MessageDialogKind::Info)
                                .show(|_| {});
                        }
                        Err(error) => {
                            app_handle
                                .dialog()
                                .message(&format!("Failed to reset temporary URL: {}", error))
                                .title("Web Server Error")
                                .kind(MessageDialogKind::Error)
                                .show(|_| {});
                        }
                    }
                });
        })
        .build(app)?;
    Ok(())
}
