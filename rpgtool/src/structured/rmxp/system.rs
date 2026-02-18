pub use crate::structured::AudioFile;
use crate::structured::NilPadded;

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::System")]
pub struct System {
    pub magic_number: i32,
    pub party_members: Vec<usize>,
    pub elements: Vec<String>, // not nil padded (for some reason)
    pub switches: NilPadded<String>,
    pub variables: NilPadded<String>,
    pub windowskin_name: String,
    pub title_name: String,
    pub gameover_name: String,
    pub battle_transition: String,
    pub title_bgm: AudioFile,
    pub battle_bgm: AudioFile,
    pub battle_end_me: AudioFile,
    pub gameover_me: AudioFile,
    pub cursor_se: AudioFile,
    pub decision_se: AudioFile,
    pub cancel_se: AudioFile,
    pub buzzer_se: AudioFile,
    pub equip_se: AudioFile,
    pub shop_se: AudioFile,
    pub save_se: AudioFile,
    pub load_se: AudioFile,
    pub battle_start_se: AudioFile,
    pub escape_se: AudioFile,
    pub actor_collapse_se: AudioFile,
    pub enemy_collapse_se: AudioFile,
    pub words: Words,
    #[serde(skip)]
    // #[marshal(skip)]
    pub test_battlers: alox_48::Value,
    pub test_troop_id: usize,
    pub start_map_id: usize,
    pub start_x: i32,
    pub start_y: i32,
    pub battleback_name: String,
    pub battler_name: String,
    pub battler_hue: i32,
    pub edit_map_id: usize,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::System::Words")]
#[serde(default)]
pub struct Words {
    pub gold: String,
    pub hp: String,
    pub sp: String,
    pub str: String,
    pub dex: String,
    pub agi: String,
    pub int: String,
    pub atk: String,
    pub pdef: String,
    pub mdef: String,
    pub weapon: String,
    pub armor1: String,
    pub armor2: String,
    pub armor3: String,
    pub armor4: String,
    pub attack: String,
    pub skill: String,
    pub guard: String,
    pub item: String,
    pub equip: String,
}

#[allow(dead_code)]
// TODO ???
#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::System::TestBattler")]
pub struct TestBattler {
    level: i32,
    actor_id: usize,
    weapon_id: usize,
    armor1_id: usize,
    armor2_id: usize,
    armor3_id: usize,
    armor4_id: usize,
}
