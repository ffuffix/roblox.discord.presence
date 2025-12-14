use rfd::MessageDialog;
use crate::util::settings::Settings;

pub fn show_settings_dialog() -> Result<Settings, Box<dyn std::error::Error>> {
    // For now simple message dialogs
    // In the future, this could be replaced with a more sophisticated GUI

    let current_settings = Settings::load();
    
    let message = format!(
        "Roblox Discord Presence Settings\n\n\
        Auto Start: {}\n\
        Show Console: {}\n\n\
        To modify settings, please edit:\n\
        {}\n\n\
        Or restart the application to load updated settings.",
        current_settings.auto_start,
        current_settings.show_console,
        Settings::config_path().display()
    );

    MessageDialog::new()
        .set_title("Settings")
        .set_description(&message)
        .show();

    Ok(current_settings)
}

pub fn show_about_dialog() {
    MessageDialog::new()
        .set_title("About")
        .set_description(
            "Roblox Discord Presence\n\
            Version 0.1.0\n\n\
            Displays your Roblox activity on Discord"
        )
        .show();
}
