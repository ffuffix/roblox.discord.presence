#![windows_subsystem = "windows"]

mod util;

use util::{
    discord::DiscordClient,
    log_parser::LogMonitor,
    paths::RobloxType,
    watcher::{self, WatcherEvent},
    roblox_api,
    notifier,
    settings::Settings,
    tray,
};

use tokio::time::{interval, Duration};
use tao::event_loop::{EventLoop, ControlFlow};
use tray_icon::menu::MenuEvent;
use auto_launch::AutoLaunchBuilder;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;

fn main() {
    let event_loop = EventLoop::new();
    
    let mut settings = Settings::load();

    if settings.show_console {
        unsafe { let _ = AllocConsole(); }
    }
    
    let mut tray_handles = Some(tray::setup_tray(&settings));
    
    let auto = AutoLaunchBuilder::new()
        .set_app_name("Roblox Discord Presence")
        .set_app_path(std::env::current_exe().unwrap().to_str().unwrap())
        .build()
        .unwrap();

    if settings.auto_start {
        let _ = auto.enable();
    } else {
        let _ = auto.disable();
    }

    // Config watcher
    let (config_tx, config_rx) = channel();
    let config_path = Settings::config_path();
    let config_path_clone = config_path.clone();
    
    let mut watcher = RecommendedWatcher::new(move |res: Result<notify::Event, _>| {
        if let Ok(event) = res {
            // Check if the event is related to our config file
            let is_config_change = event.paths.iter().any(|p| p == &config_path_clone);
            if is_config_change {
                let _ = config_tx.send(());
            }
        }
    }, Config::default()).unwrap();
    
    if let Some(parent) = config_path.parent() {
        let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
    }
    
    // Spawn async runtime
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async_main());
    });

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(50));

        if let Ok(_) = config_rx.try_recv() {
            println!("Config changed, reloading settings...");
            let new_settings = Settings::load();
            
            // Only update if settings actually changed
            if new_settings.auto_start != settings.auto_start {
                settings.auto_start = new_settings.auto_start;
                if let Some(handles) = &tray_handles {
                    handles.auto_start.set_checked(settings.auto_start);
                }
                if settings.auto_start {
                    let _ = auto.enable();
                } else {
                    let _ = auto.disable();
                }
            }

            if new_settings.show_console != settings.show_console {
                settings.show_console = new_settings.show_console;
                if let Some(handles) = &tray_handles {
                    handles.show_console.set_checked(settings.show_console);
                }
                unsafe {
                    if settings.show_console {
                        let _ = AllocConsole();
                    } else {
                        let _ = FreeConsole();
                    }
                }
            }
            // Update other settings if needed
            // settings = new_settings; // Or just replace the whole struct if it were simple, but we handle side effects above
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id.as_ref() {
                tray::MENU_QUIT_ID => {
                    *control_flow = ControlFlow::Exit;
                    std::process::exit(0);
                }
                tray::MENU_OPEN_CONFIG_ID => {
                    let path = Settings::config_path();
                    if let Some(parent) = path.parent() {
                        if !parent.exists() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                    }
                    // Ensure the settings file exists before opening
                    if !path.exists() {
                        if let Err(e) = settings.save() {
                            notifier::error("Config Error", &format!("Failed to create config file: {}", e));
                        }
                    }
                    // Open the config file with the default editor
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("cmd")
                            .args(&["/C", "start", "", &path.to_string_lossy()])
                            .spawn();
                    }
                    
                    #[cfg(target_os = "macos")]
                    {
                        let _ = std::process::Command::new("open")
                            .arg(&path)
                            .spawn();
                    }
                    
                    #[cfg(target_os = "linux")]
                    {
                        let _ = std::process::Command::new("xdg-open")
                            .arg(&path)
                            .spawn();
                    }
                    
                    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
                    {
                        notifier::error("Config Error", &format!("Opening config file is not supported on this platform. Config file location: {}", path.display()));
                    }
                }
                tray::MENU_AUTO_START_ID => {
                    settings.auto_start = !settings.auto_start;
                    let _ = settings.save();
                    if let Some(handles) = &tray_handles {
                        handles.auto_start.set_checked(settings.auto_start);
                    }
                    
                    if settings.auto_start {
                        if let Err(e) = auto.enable() {
                            notifier::error("Auto Start Error", &format!("Failed to enable: {}", e));
                            settings.auto_start = false;
                            if let Some(handles) = &tray_handles {
                                handles.auto_start.set_checked(false);
                            }
                            let _ = settings.save();
                        }
                    } else {
                        if let Err(e) = auto.disable() {
                            notifier::error("Auto Start Error", &format!("Failed to disable: {}", e));
                        }
                    }
                }
                tray::MENU_SHOW_CONSOLE_ID => {
                    settings.show_console = !settings.show_console;
                    let _ = settings.save();
                    
                    if let Some(handles) = &tray_handles {
                        handles.show_console.set_checked(settings.show_console);
                    }

                    unsafe {
                        if settings.show_console {
                            let _ = AllocConsole();
                        } else {
                            let _ = FreeConsole();
                        }
                    }
                }
                _ => {}
            }
        }
    });
}

async fn async_main() {
    let mut discord_client = DiscordClient::new();
    let mut log_monitor = LogMonitor::new();
    
    let mut current_roblox_type: Option<RobloxType> = None;
    let mut last_place_id: String = String::new();

    let mut event_receiver = watcher::spawn_watcher();
    
    let mut log_poll_interval = interval(Duration::from_secs(2));
    log_poll_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    
    loop {
        tokio::select! {
            event = event_receiver.recv() => {
                match event {
                    Some(WatcherEvent::RobloxStarted(rt)) => {
                        last_place_id.clear(); 
                        current_roblox_type = Some(rt);

                        if rt == RobloxType::Studio {
                            discord_client.update_presence("Roblox Studio", "Developing", "roblox_studio", None);
                        } else {
                            discord_client.update_presence("Roblox", "Loading", "roblox_logo", None);
                        }
                    }
                    Some(WatcherEvent::RobloxClosed) => {
                        discord_client.clear_presence();
                        log_monitor.clear();
                        current_roblox_type = None;
                        last_place_id.clear();
                    }
                    None => {
                        break;
                    }
                }
            }

            _ = log_poll_interval.tick() => {
                if let Some(roblox_type) = current_roblox_type {
                    if let Some(id) = log_monitor.check_latest_log() {
                        if id != last_place_id {
                            last_place_id = id.clone();

                            match roblox_api::get_game_details(&id).await {
                                Ok(details) => {
                                    match roblox_type {
                                        RobloxType::Player => {
                                            let state_str = format!("by {}", details.creator_name);
                                            let stats_str = format!("Playing: {} | Capacity: {}", format_num(details.playing), details.max_players);
                                            
                                            discord_client.update_presence(
                                                &details.name, 
                                                &state_str,
                                                &details.thumbnail_url,
                                                Some(&stats_str) 
                                            );
                                        },
                                        RobloxType::Studio => {
                                            let state_str = "Editing";

                                            discord_client.update_presence(
                                                &details.name, 
                                                state_str,
                                                &details.thumbnail_url,
                                                Some("Developing")
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    notifier::error("Game Details Error", &format!("Failed to fetch details: {}", e));
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_num(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;
    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }
    result.chars().rev().collect()
}
