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

fn main() {
    let event_loop = EventLoop::new();
    
    let mut settings = Settings::load();
    
    let tray_handles = tray::setup_tray(&settings);
    
    let auto = AutoLaunchBuilder::new()
        .set_app_name("Roblox Discord Presence")
        .set_app_path(std::env::current_exe().unwrap().to_str().unwrap())
        .set_use_launch_agent(true)
        .build()
        .unwrap();

    if settings.auto_start {
        let _ = auto.enable();
    } else {
        let _ = auto.disable();
    }
    
    // Spawn async runtime
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async_main());
    });

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(50));

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
                        let _ = settings.save();
                    }
                    // Open the config file with the default editor
                    #[cfg(target_os = "windows")]
                    let _ = std::process::Command::new("cmd")
                        .args(&["/C", "start", "", path.to_str().unwrap_or("")])
                        .spawn();
                    
                    #[cfg(target_os = "macos")]
                    let _ = std::process::Command::new("open")
                        .arg(&path)
                        .spawn();
                    
                    #[cfg(target_os = "linux")]
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&path)
                        .spawn();
                }
                tray::MENU_AUTO_START_ID => {
                    settings.auto_start = !settings.auto_start;
                    let _ = settings.save();
                    tray_handles.auto_start.set_checked(settings.auto_start);
                    
                    if settings.auto_start {
                        let _ = auto.enable();
                    } else {
                        let _ = auto.disable();
                    }
                }
                tray::MENU_SHOW_CONSOLE_ID => {
                    settings.show_console = !settings.show_console;
                    let _ = settings.save();
                    tray_handles.show_console.set_checked(settings.show_console);
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
