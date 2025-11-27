use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_ID: &str = "1442858852730277890";

pub struct DiscordClient {
    client: Option<DiscordIpcClient>,
    start_time: u64,
}

impl DiscordClient {
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        let client = match DiscordIpcClient::new(APP_ID) {
            Ok(mut c) => {
                if c.connect().is_ok() {
                    println!("Connected to Discord");
                    Some(c)
                } else {
                    eprintln!("Failed to connect to Discord (is discord open?)");
                    None
                }
            },
            Err(e) => {
                eprintln!("Failed to create Discord client: {}", e);
                None
            }
        };

        Self { client, start_time }
    }

    pub fn update_presence(
        &mut self, 
        details: &str, 
        state: &str, 
        large_image: &str, 
        small_text: Option<&str>
    ) {
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
                eprintln!("Failed to set activity: {}", e);
            }
        }
    }

    pub fn clear_presence(&mut self) {
        if let Some(client) = self.client.as_mut() {
            if let Err(e) = client.clear_activity() {
                eprintln!("Failed to clear activity: {}", e);
            }
        }
    }

    pub fn close(&mut self) {
        if let Some(client) = self.client.as_mut() {
            let _ = client.close();
        }
    }
}

impl Drop for DiscordClient {
    fn drop(&mut self) {
        self.close();
    }
}