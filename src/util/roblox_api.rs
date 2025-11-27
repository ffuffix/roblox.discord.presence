use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UniverseIdResponse {
    pub universe_id: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub name: String,
    pub playing: u64,
    pub max_players: u64,
    pub creator: CreatorInfo,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatorInfo {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct GamesResponse {
    pub data: Vec<GameInfo>,
}

#[derive(Deserialize, Debug)]
pub struct ThumbnailInfo {
    pub state: String,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
}

#[derive(Deserialize, Debug)]
pub struct ThumbnailsResponse {
    pub data: Vec<ThumbnailInfo>,
}

#[derive(Debug)]
pub struct GameDetails {
    pub name: String,
    pub thumbnail_url: String,
    pub playing: u64,
    pub max_players: u64,
    pub creator_name: String,
}

pub async fn get_game_details(place_id: &str) -> Result<GameDetails, reqwest::Error> {
    let client = reqwest::Client::new();

    let universe_url = format!("https://apis.roblox.com/universes/v1/places/{}/universe", place_id);
    let universe_res = client.get(&universe_url).send().await?;
    let universe_data: UniverseIdResponse = universe_res.json().await?;
    let universe_id = universe_data.universe_id;

    let game_url = format!("https://games.roblox.com/v1/games?universeIds={}", universe_id);
    let game_res = client.get(&game_url).send().await?;
    let game_body: GamesResponse = game_res.json().await?;

    if game_body.data.is_empty() {
        return Ok(GameDetails {
            name: "Unknown Game".to_string(),
            thumbnail_url: "roblox_logo".to_string(),
            playing: 0,
            max_players: 0,
            creator_name: "Unknown".to_string(),
        });
    }

    let game_info = &game_body.data[0];

    let thumb_url = format!("https://thumbnails.roblox.com/v1/games/icons?universeIds={}&size=512x512&format=Png&isCircular=false", universe_id);
    let thumb_res = client.get(&thumb_url).send().await?;
    let thumb_body: ThumbnailsResponse = thumb_res.json().await?;

    let thumbnail_url = if !thumb_body.data.is_empty() && thumb_body.data[0].state == "Completed" {
        thumb_body.data[0].image_url.clone()
    } else {
        "roblox_logo".to_string()
    };
    
    Ok(GameDetails {
        name: game_info.name.clone(),
        thumbnail_url,
        playing: game_info.playing,
        max_players: game_info.max_players,
        creator_name: game_info.creator.name.clone(),
    })
}