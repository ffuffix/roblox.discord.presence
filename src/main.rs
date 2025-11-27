mod util;

use util::{
    discord::DiscordClient,
    log_parser::LogMonitor,
    paths::RobloxType,
    watcher::{self, WatcherEvent},
    roblox_api,
};

use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    let mut discord_client = DiscordClient::new();
    let mut log_monitor = LogMonitor::new();
    
    let mut current_roblox_type: Option<RobloxType> = None;
    let mut last_place_id: String = String::new();

    let mut event_receiver = watcher::spawn_watcher();
    
    let mut log_poll_interval = interval(Duration::from_secs(2));
    log_poll_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    println!("The Watcher is watching");

    loop {
        tokio::select! {
            // 1. Handle Process Events (Started/Stopped)
            event = event_receiver.recv() => {
                match event {
                    Some(WatcherEvent::RobloxStarted(rt)) => {
                        current_roblox_type = Some(rt);
                        if rt == RobloxType::Studio {
                            println!("Roblox Studio detected");
                            discord_client.update_presence("Roblox Studio", "Developing", "roblox_studio", None);
                        } else {
                            println!("Roblox Player detected");
                            discord_client.update_presence("Roblox", "Loading...", "roblox_logo", None);
                        }
                    }
                    Some(WatcherEvent::RobloxClosed) => {
                        println!("Roblox closed");
                        discord_client.clear_presence();
                        log_monitor.clear();
                        current_roblox_type = None;
                        last_place_id.clear();
                    }
                    None => {
                        println!("Watcher channel closed. Exiting");
                        break;
                    }
                }
            }

            _ = log_poll_interval.tick() => {
                if current_roblox_type == Some(RobloxType::Player) {
                    if let Some(id) = log_monitor.check_latest_log() {
                        if id != last_place_id {
                            println!("> Detected Place ID: {}", id);
                            last_place_id = id.clone();

                            match roblox_api::get_game_details(&id).await {
                                Ok(details) => {
                                    println!("> Fetched: {} by {}", details.name, details.creator_name);
                                    let state_str = format!("by {}", details.creator_name);
                                    let stats_str = format!("Playing: {} | Capacity: {}", format_num(details.playing), details.max_players);

                                    discord_client.update_presence(
                                        &details.name, 
                                        &state_str,
                                        &details.thumbnail_url,
                                        Some(&stats_str) 
                                    );
                                }
                                Err(e) => eprintln!("Failed to get game details: {}", e),
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