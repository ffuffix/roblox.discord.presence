# Roblox Discord Presence

A simple application that integrates Roblox game activity with Discord Rich Presence using Rust!

### Installation
1. Download the latest release from the [Releases](https://github.com/ffuffix/roblox.discord.presence/releases).
2. Run the executable file (`roblox_discord_presence.exe` on Windows).


### Usage
1. Launch the application
2. The application runs in the system tray (look for the red circle icon)
3. Right-click the tray icon to access the menu with the following options:
   - **Auto Start**: Enable/disable automatic startup with your system
   - **Show Console**: Toggle console window visibility (for debugging)
   - **Open Config File**: Open the settings file in your default text editor
   - **Quit**: Exit the application
4. Start playing a Roblox game or open Roblox Studio
5. Your Discord status automatically updates to reflect your activity

### Settings

The application stores its settings in a configuration file (`settings.toml`) which is automatically created on first run. The file location varies by platform:

- **Windows**: `%APPDATA%\roblox_discord_presence\settings.toml`
- **macOS**: `~/Library/Application Support/roblox_discord_presence/settings.toml`
- **Linux**: `~/.config/roblox_discord_presence/settings.toml`

You can access the settings file directly by clicking "Open Config File" in the system tray menu. The settings include:

- `auto_start`: Automatically start the application when your system boots
- `show_console`: Show or hide the console window (useful for debugging)
- `custom_status_template`: (Reserved for future use) Custom template for Discord status

Changes made through the system tray menu are automatically saved. If you manually edit the settings file, restart the application to apply the changes.


### Building from Source

1. Ensure you have Rust installed. You can download it from [rust-lang.org](https://www.rust-lang.org/tools/install)
2. Clone this repository:

```bash
git clone https://github.com/ffuffix/roblox.discord.presence
```

3. Navigate to the project directory:

```bash
cd roblox.discord.presence
```

4. Build the project using Cargo:

```bash
cargo build --release
```

5. The compiled binary will be located in the `target/release` directory!

6. Run it:

```bash
./target/release/roblox_discord_presence.exe
```


### Contributing
Contributions are welcome! Feel free to open issues or submit pull requests for improvements and new features