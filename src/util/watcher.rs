use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use sysinfo::{System, ProcessRefreshKind, RefreshKind}; 

use super::paths::RobloxType;

#[derive(Debug)]
pub enum WatcherEvent {
    RobloxStarted(RobloxType),
    RobloxClosed,
}

pub fn spawn_watcher() -> UnboundedReceiver<WatcherEvent> {
    let (tx, rx) = unbounded_channel();

    std::thread::spawn(move || {
        // Only refresh processes, saves CPU
        let mut system = System::new_with_specifics(
            RefreshKind::new().with_processes(ProcessRefreshKind::everything())
        );

        let mut roblox_was_running: Option<RobloxType> = None;

        loop {
            system.refresh_processes();

            let roblox_is_running = get_running_roblox_type(&system);

            if roblox_is_running.is_some() && roblox_was_running.is_none() {
                let rtype = roblox_is_running.unwrap();
                println!("[WATCHER] Process Started: {:?}", rtype);
                let _ = tx.send(WatcherEvent::RobloxStarted(rtype));
            } else if roblox_is_running.is_none() && roblox_was_running.is_some() {
                println!("[WATCHER] Process Closed");
                let _ = tx.send(WatcherEvent::RobloxClosed);
            }

            roblox_was_running = roblox_is_running;
            std::thread::sleep(Duration::from_secs(1));
        }
    });

    rx
}

fn get_running_roblox_type(system: &System) -> Option<RobloxType> {
    for (_pid, process) in system.processes() {
        let name = process.name().to_lowercase();
        
        if name.contains("robloxstudio") {
            return Some(RobloxType::Studio);
        }

        if name.contains("robloxplayer") {
            return Some(RobloxType::Player);
        }
    }

    None
}