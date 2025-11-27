use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RobloxType {
    Player,
    Studio,
}

pub fn roblox() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    
    #[cfg(target_os = "windows")]
    let path = home.join("AppData/Local/Roblox");

    #[cfg(target_os = "macos")]
    let path = home.join("Library/Application Support/Roblox");
    
    #[cfg(target_os = "linux")]
    let path = home.join(".local/share/Roblox");
    
    Some(path)
}

pub fn roblox_logs() -> Option<PathBuf> {
    let roblox_path = roblox()?;
    let path = roblox_path.join("logs");
    Some(path)
}