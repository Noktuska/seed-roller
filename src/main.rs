use std::path::Path;

use anyhow::{Result, bail};
use clap::Parser;
use maprando::{difficulty::{get_full_global, get_link_difficulty_length}, map_repository::MapRepository, preset::PresetData, randomize::{Randomizer, assign_map_areas, filter_links, get_difficulty_tiers, get_objectives, randomize_doors}, settings::RandomizerSettings, spoiler_log::SpoilerLog};
use maprando_game::{GameData, LinksDataGroup, Map};
use mlua::{Function, Lua, LuaSerdeExt, SerializeOptions};
use rand::{RngCore, SeedableRng, rngs::StdRng};
use serde_json::Value;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    random_seed: Option<usize>,
    #[arg(long)]
    settings_path: String,
    #[arg(long)]
    lua_path: Option<String>,

    #[arg(long)]
    map_layout: Option<String>,

    #[arg(long, default_value_t = 100)]
    attempts: usize,
    #[arg(long, default_value_t = 100)]
    attempts_per_seed: usize,
    #[arg(long, default_value_t = 10)]
    attempts_per_map: usize,

    #[arg(long, default_value_t = 1)]
    threads: usize,

    #[arg(long)]
    stop_on_success: bool
}

fn main() -> Result<()> {
    let args = Args::parse();

    let tech_path = Path::new("./data/maprando-data/data/tech_data.json");
    let notable_path = Path::new("./data/maprando-data/data/notable_data.json");
    let presets_path = Path::new("./data/maprando-data/data/presets/");

    println!("Loading Game Data...");

    let game_data_path = Path::new("./data/maprando-data/");
    let mut game_data = GameData::load(game_data_path)?;
    let preset_data = PresetData::load(tech_path, notable_path, presets_path, &game_data)?;
    let global = get_full_global(&game_data);
    game_data.make_links_data(&|link, game_data| {
        get_link_difficulty_length(link, game_data, &preset_data, &global)
    });

    println!("Loading Map Repositories...");

    let path_vanilla = Path::new("./data/maps/vanilla/");
    let path_standard = Path::new("./data/maps/v119-standard-avro/");
    let path_wild = Path::new("./data/maps/v119-wild-avro/");
    let map_repo_vanilla = MapRepository::new("Vanilla", path_vanilla)?;
    let map_repo_standard = MapRepository::new("Standard", path_standard)?;
    let map_repo_wild = MapRepository::new("Wild", path_wild)?;

    println!("Loading Randomizer Settings...");

    let settings_str = std::fs::read_to_string(&args.settings_path)?;
    let settings: RandomizerSettings = serde_json::from_str(&settings_str)?;

    let implicit_tech = &preset_data.tech_by_difficulty["Implicit"];
    let implicit_notables = &preset_data.notables_by_difficulty["Implicit"];
    let difficulty_tiers = get_difficulty_tiers(&settings, &preset_data.difficulty_tiers, &game_data, implicit_tech, implicit_notables);

    let filtered_base_links = filter_links(&game_data.links, &game_data, &difficulty_tiers[0]);
    let filtered_base_links_data = LinksDataGroup::new(filtered_base_links, game_data.vertex_isv.keys.len(), 0);
    
    let map_layout = args.map_layout.clone().unwrap_or("Vanilla".to_string());
    let map_repo = match map_layout.to_ascii_lowercase().as_str() {
        "vanilla" => &map_repo_vanilla,
        "standard" => &map_repo_standard,
        "wild" => &map_repo_wild,
        _ => bail!("Invalid Map repository: {map_layout}")
    };

    let max_attempts = args.attempts_per_seed;
    let max_attempts_per_map = args.attempts_per_map;
    let max_map_attempts = max_attempts / max_attempts_per_map;

    let attempts = if args.random_seed.is_some() {
        1
    } else {
        args.attempts
    };

    println!("Loading Lua Script...");

    let lua = Lua::new();
    let lua_script = if let Some(lua_path) = &args.lua_path {
        let lua_src = std::fs::read_to_string(&lua_path)?;
        Some(lua.load(lua_src).eval::<Function>()?)
    } else {
        None
    };

    let options = SerializeOptions::default()
        .serialize_none_to_null(false)
        .serialize_unit_to_null(false)
        .set_array_metatable(false);

    let mut best_spoiler_log: Option<SpoilerLog> = None;
    let mut best_random_seed = 0;

    println!("Rolling seeds...");

    'reroll_seed: for i in 0..attempts {
        let random_seed = args.random_seed.unwrap_or_else(get_random_seed);
        let mut rng_seed = [0u8; 32];
        rng_seed[..8].copy_from_slice(&random_seed.to_le_bytes());
        let mut rng = StdRng::from_seed(rng_seed);

        let mut attempt_num = 0;
        let mut map_batch: Vec<Map> = vec![];

        println!("Reroll seed {i}/{}, seed: {random_seed}", attempts);

        for _ in 0..max_map_attempts {
            let map_seed = (rng.next_u64() & 0xFFFFFFFF) as usize;
            let door_randomization_seed = (rng.next_u64() & 0xFFFFFFFF) as usize;

            if map_batch.is_empty() {
                map_batch = map_repo.get_map_batch(map_seed, &game_data)?;
            }
            let mut map = map_batch.pop().unwrap();

            if !assign_map_areas(&mut map, &settings, map_seed, &game_data) {
                continue;
            }

            let objectives = get_objectives(&settings, Some(&map), &game_data, &mut rng);
            let locked_door_data = randomize_doors(&game_data, &map, &settings, &objectives, door_randomization_seed);
            let randomizer = Randomizer::new(
                &map,
                &locked_door_data,
                objectives.clone(),
                &settings,
                &difficulty_tiers,
                &game_data,
                &filtered_base_links_data,
                &mut rng
            );

            for _ in 0..max_attempts_per_map {
                let item_placement_seed = (rng.next_u64() & 0xFFFFFFFF) as usize;
                attempt_num += 1;

                //println!("Seed attempt {attempt_num}/{max_attempts}");

                let randomization_result = randomizer.randomize(attempt_num, item_placement_seed, random_seed, true);
                let Ok((_r, s)) = randomization_result else {
                    continue;
                };

                println!("Successful attempt {attempt_num}/{max_attempts}");

                if let Some(script) = lua_script.as_ref() {
                    let new_spoiler = lua.to_value_with(&s, options)?;
                    let old_spoiler = if let Some(best_spoiler) = &best_spoiler_log {
                        lua.to_value_with(best_spoiler, options)?
                    } else {
                        mlua::Value::Nil
                    };
                    if script.call::<bool>((new_spoiler, old_spoiler))? {
                        best_spoiler_log = Some(s);
                        best_random_seed = random_seed;
                        if args.stop_on_success {
                            break 'reroll_seed;
                        }
                    }
                } else if best_spoiler_log.is_none() {
                    best_spoiler_log = Some(s);
                    best_random_seed = random_seed;
                    break 'reroll_seed;
                }

                continue 'reroll_seed;
            }
        }
    }

    if let Some(s) = best_spoiler_log {
        let mut spoiler_v: Value = serde_json::to_value(&s)?;
        let spoiler_obj = spoiler_v.as_object_mut().unwrap();
        spoiler_obj.remove("game_data");
        spoiler_obj.remove("forward_traversal");
        spoiler_obj.remove("reverse_traversal");

        let spoiler_str = serde_json::to_string_pretty(&spoiler_v)?;
        let out_path = format!("spoiler_{best_random_seed}.json");
        std::fs::write(&out_path, &spoiler_str)?;

        println!("Written Spoiler log to file {out_path}");
    }

    Ok(())
}

fn get_random_seed() -> usize {
    (StdRng::from_entropy().next_u64() & 0xFFFFFFFF) as usize + 1
}