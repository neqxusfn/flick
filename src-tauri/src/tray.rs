use crate::engine::{Engine, Snapshot};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
pub struct TrayItems {
    enabled: MenuItem<tauri::Wry>,
    profile: MenuItem<tauri::Wry>,
}
pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let enabled = MenuItem::with_id(app, "toggle", "Enable Flick", true, None::<&str>)?;
    let profile = MenuItem::with_id(app, "profile", "My profile", false, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open Flick", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&enabled, &profile, &open, &quit])?;
    let mut builder = TrayIconBuilder::with_id("flick")
        .tooltip("Flick · Paused")
        .menu(&menu)
        .show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => {
                let engine = app.state::<Arc<Engine>>();
                engine.set_enabled(!engine.snapshot().enabled);
            }
            "open" => show(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|icon, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show(icon.app_handle());
            }
        })
        .build(app)?;
    app.manage(TrayItems { enabled, profile });
    Ok(())
}
pub fn refresh(app: &AppHandle, state: &Snapshot) {
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.enabled.set_text(if state.enabled {
            "Disable Flick"
        } else {
            "Enable Flick"
        });
        let _ = items
            .profile
            .set_text(format!("Profile: {}", state.settings.active().name));
    }
    if let Some(icon) = app.tray_by_id("flick") {
        let _ = icon.set_tooltip(Some(if state.enabled {
            "Flick · Enabled"
        } else {
            "Flick · Paused"
        }));
    }
}
