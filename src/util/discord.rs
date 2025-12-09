use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_ID: &str = "1442858852730277890";

pub struct DiscordClient {
    client: Option<DiscordIpcClient>,
    connected: bool,
    start_time: u64,
}

impl DiscordClient {
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let client = match DiscordIpcClient::new(APP_ID) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Failed to initialize Discord IPC client structure: {}", e);
                None
            }
        };

        Self {
            client,
            connected: false,
            start_time,
        }
    }

    fn ensure_connected(&mut self) -> bool {
        if self.connected {
            return true;
        }

        if let Some(client) = self.client.as_mut() {
            match client.connect() {
                Ok(_) => {
                    println!("Connected to Discord");
                    self.connected = true;
                    return true;
                }
                Err(_) => {
                    return false;
                }
            }
        }

        false
    }

    fn reset_start_time(&mut self) {
        self.start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
    }

    pub fn update_presence(
        &mut self,
        details: &str,
        state: &str,
        large_image: &str,
        small_text: Option<&str>,
    ) {
        if !self.ensure_connected() {
            return;
        }

        self.reset_start_time();

        if let Some(client) = self.client.as_mut() {
            let mut assets = activity::Assets::new()
                .large_image(large_image)
                .large_text(details);

            if let Some(txt) = small_text {
                assets = assets.small_image("roblox_logo").small_text(txt);
            }

            let timestamps = activity::Timestamps::new().start(self.start_time as i64);

            let activity = activity::Activity::new()
                .details(details)
                .state(state)
                .assets(assets)
                .timestamps(timestamps);

            if let Err(e) = client.set_activity(activity) {
                eprintln!("Failed to set activity (Discord might have closed): {}", e);
                self.connected = false;
                let _ = client.close();
            }
        }
    }

    pub fn clear_presence(&mut self) {
        if self.connected {
            if let Some(client) = self.client.as_mut() {
                if let Err(_) = client.clear_activity() {
                    self.connected = false;
                    let _ = client.close();
                }
            }
        }
    }

    pub fn close(&mut self) {
        if let Some(client) = self.client.as_mut() {
            let _ = client.close();
        }
        self.connected = false;
    }
}

impl Drop for DiscordClient {
    fn drop(&mut self) {
        self.close();
    }
}
