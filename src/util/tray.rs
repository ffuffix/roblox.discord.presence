use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem},
    TrayIcon, TrayIconBuilder, Icon,
};

pub const MENU_QUIT_ID: &str = "quit";
pub const MENU_AUTO_START_ID: &str = "auto_start";
pub const MENU_SHOW_CONSOLE_ID: &str = "show_console";
pub const MENU_OPEN_CONFIG_ID: &str = "open_config";

pub struct TrayHandles {
    pub tray_icon: TrayIcon,
    pub auto_start: CheckMenuItem,
    pub show_console: CheckMenuItem,
}

pub fn setup_tray(settings: &crate::util::settings::Settings) -> TrayHandles {
    let icon = create_default_icon();
    
    let tray_menu = Menu::new();
    
    let auto_start = CheckMenuItem::with_id(MENU_AUTO_START_ID, "Auto Start", settings.auto_start, true, None);
    let show_console = CheckMenuItem::with_id(MENU_SHOW_CONSOLE_ID, "Show Console", settings.show_console, true, None);
    let open_config = MenuItem::with_id(MENU_OPEN_CONFIG_ID, "Open Config File", true, None);
    let quit = MenuItem::with_id(MENU_QUIT_ID, "Quit", true, None);
    
    tray_menu.append_items(&[
        &auto_start,
        &show_console,
        &PredefinedMenuItem::separator(),
        &open_config,
        &PredefinedMenuItem::separator(),
        &quit,
    ]).unwrap();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu.clone()))
        .with_tooltip("Roblox Discord Presence")
        .with_icon(icon)
        .build()
        .unwrap();

    TrayHandles {
        tray_icon,
        auto_start,
        show_console,
    }
}

fn create_default_icon() -> Icon {
    let width = 64;
    let height = 64;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    
    for i in 0..width {
        for j in 0..height {
            // Simple red circle
            let dx = i as i32 - 32;
            let dy = j as i32 - 32;
            if dx * dx + dy * dy < 28 * 28 {
                rgba.extend_from_slice(&[220, 50, 50, 255]); 
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]); 
            }
        }
    }
    
    Icon::from_rgba(rgba, width, height).unwrap_or_else(|_| {
        Icon::from_rgba(vec![0, 0, 0, 0], 1, 1).unwrap()
    })
}
