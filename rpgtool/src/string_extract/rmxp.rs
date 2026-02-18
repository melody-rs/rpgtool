use super::{Format, GameString, ProcessResult, read_data};
#[allow(clippy::wildcard_imports)]
use crate::structured::*;

pub fn process(
    path: &std::path::Path,
    format: Format,
    strings: &mut Vec<GameString>,
) -> ProcessResult {
    let prefix = path.file_prefix().expect("there should be a prefix");
    let Some(filename) = prefix.to_str() else {
        return ProcessResult::Err(format!("{} is not valid UTF-8", prefix.display()));
    };

    let result = match filename {
        "Actors" => read_data(path, format).map(|v| process_actors(v, strings)),
        "Animations" => read_data(path, format).map(|v| process_animations(v, strings)),
        "Armors" => read_data(path, format).map(|v| process_armors(v, strings)),
        "Classes" => read_data(path, format).map(|v| process_classes(v, strings)),
        "Enemies" => read_data(path, format).map(|v| process_enemies(v, strings)),
        "Items" => read_data(path, format).map(|v| process_items(v, strings)),
        "Skills" => read_data(path, format).map(|v| process_skills(v, strings)),
        "States" => read_data(path, format).map(|v| process_states(v, strings)),
        "System" => read_data(path, format).map(|v| process_system(v, strings)),
        "Tilesets" => read_data(path, format).map(|v| process_tilesets(v, strings)),
        "Troops" => read_data(path, format).map(|v| process_troops(v, strings)),
        "Weapons" => read_data(path, format).map(|v| process_weapons(v, strings)),
        "MapInfos" => read_data(path, format).map(|v| process_mapinfos(v, strings)),
        "CommonEvents" => read_data(path, format).map(|v| process_commonevents(v, strings)),
        _ if filename.starts_with("Map") => {
            #[allow(clippy::unwrap_used)] // TODO
            let map_id = filename.strip_prefix("Map").unwrap().parse().unwrap();
            read_data(path, format).map(|v| process_map(v, map_id, strings))
        }
        "Scripts" | "xScripts" => read_data(path, format).map(|v| process_scripts(v, strings)),
        _ => return ProcessResult::Unrecognized,
    };

    match result {
        Ok(()) => ProcessResult::Ok,
        Result::Err(e) => ProcessResult::Err(e),
    }
}

fn add_string(strings: &mut Vec<GameString>, text: String, location: String) {
    // only add string if there is actual text and not just whitespace
    if text.split_whitespace().next().is_some() {
        strings.push(GameString { location, text });
    }
}

fn process_actors(actors: rmxp::Actors, strings: &mut Vec<GameString>) {
    for (i, actor) in actors.0.into_iter().enumerate() {
        add_string(strings, actor.name, format!("Actor:{i}:name"));
    }
}

fn process_animations(anims: rmxp::Animations, strings: &mut Vec<GameString>) {
    for (i, anim) in anims.0.into_iter().enumerate() {
        add_string(strings, anim.name, format!("Animation:{i}:name"));
    }
}

fn process_armors(armors: rmxp::Armors, strings: &mut Vec<GameString>) {
    for (i, armor) in armors.0.into_iter().enumerate() {
        add_string(strings, armor.name, format!("Armor:{i}:name"));

        add_string(strings, armor.description, format!("Armor:{i}:description"));
    }
}

fn process_classes(classes: rmxp::Classes, strings: &mut Vec<GameString>) {
    for (i, class) in classes.0.into_iter().enumerate() {
        add_string(strings, class.name, format!("Class:{i}:name"));
    }
}

fn process_enemies(enemies: rmxp::Enemies, strings: &mut Vec<GameString>) {
    for (i, enemy) in enemies.0.into_iter().enumerate() {
        add_string(strings, enemy.name, format!("Enemy:{i}:name"));
    }
}

fn process_items(items: rmxp::Items, strings: &mut Vec<GameString>) {
    for (i, item) in items.0.into_iter().enumerate() {
        add_string(strings, item.name, format!("Item:{i}:name"));

        add_string(strings, item.description, format!("Item:{i}:description"));
    }
}

fn process_skills(skills: rmxp::Skills, strings: &mut Vec<GameString>) {
    for (i, skill) in skills.0.into_iter().enumerate() {
        add_string(strings, skill.name, format!("Skill:{i}:name"));

        add_string(strings, skill.description, format!("Skill:{i}:description"));
    }
}

fn process_states(states: rmxp::States, strings: &mut Vec<GameString>) {
    for (i, state) in states.0.into_iter().enumerate() {
        add_string(strings, state.name, format!("State:{i}:name"));
    }
}

fn process_system(system: rmxp::System, strings: &mut Vec<GameString>) {
    for (i, element) in system.elements.into_iter().enumerate() {
        add_string(strings, element, format!("Element:{i}:name"));
    }

    for (i, switch) in system.switches.0.into_iter().enumerate() {
        add_string(strings, switch, format!("Switch:{i}:name"));
    }

    for (i, variable) in system.variables.0.into_iter().enumerate() {
        add_string(strings, variable, format!("Variable:{i}:name"));
    }
}

fn process_tilesets(tilesets: rmxp::Tilesets, strings: &mut Vec<GameString>) {
    for (i, tileset) in tilesets.0.into_iter().enumerate() {
        add_string(strings, tileset.name, format!("Tileset:{i}:name"));
    }
}

fn process_troops(troops: rmxp::Troops, strings: &mut Vec<GameString>) {
    for (i, troop) in troops.0.into_iter().enumerate() {
        add_string(strings, troop.name, format!("Troop:{i}:name"));

        for (p, page) in troop.pages.into_iter().enumerate() {
            process_eventcommands(page.list, strings, &format!("Troop:{i}:page:{p}"));
        }
    }
}

fn process_weapons(weapons: rmxp::Weapons, strings: &mut Vec<GameString>) {
    for (i, weapon) in weapons.0.into_iter().enumerate() {
        add_string(strings, weapon.name, format!("Weapon:{i}:name"));

        add_string(
            strings,
            weapon.description,
            format!("Weapon:{i}:description"),
        );
    }
}

fn process_mapinfos(mapinfos: MapInfos, strings: &mut Vec<GameString>) {
    for (id, info) in mapinfos {
        add_string(strings, info.name, format!("Map:{id}:name"));
    }
}

fn process_commonevents(events: CommonEvents, strings: &mut Vec<GameString>) {
    for (i, event) in events.0.into_iter().enumerate() {
        add_string(strings, event.name, format!("CommonEvent:{i}:name"));

        process_eventcommands(event.list, strings, &format!("CommonEvent:{i}"));
    }
}

fn process_map(map: rmxp::Map, map_id: i32, strings: &mut Vec<GameString>) {
    for (id, event) in map.events {
        add_string(strings, event.name, format!("Map:{map_id}:event:{id}:name"));

        for (i, page) in event.pages.into_iter().enumerate() {
            process_eventcommands(
                page.list,
                strings,
                &format!("Map:{map_id}:event:{id}:page:{i}"),
            );

            process_move_route(
                &page.move_route,
                strings,
                &format!("Map:{map_id}:event:{id}:page:{i}:route"),
            );
        }
    }
}

fn process_scripts(scripts: Vec<Script>, strings: &mut Vec<GameString>) {
    for script in scripts {
        super::process_script_text(script.text, strings, format!("Script:{}", script.name));
    }
}

fn string_parameter_at(parameters: &[ParameterType], index: usize) -> Option<&str> {
    let ParameterType::String(text) = parameters.get(index)? else {
        return None;
    };
    Some(text)
}

fn int_parameter_at(parameters: &[ParameterType], index: usize) -> Option<i32> {
    let ParameterType::Integer(int) = parameters.get(index)? else {
        return None;
    };
    Some(*int)
}

fn route_parameter_at(parameters: &[ParameterType], index: usize) -> Option<&MoveRoute> {
    let ParameterType::MoveRoute(route) = parameters.get(index)? else {
        return None;
    };
    Some(route)
}

fn array_parameter_at(parameters: &[ParameterType], index: usize) -> Option<&[ParameterType]> {
    let ParameterType::Array(array) = parameters.get(index)? else {
        return None;
    };
    Some(array)
}

#[allow(clippy::unwrap_used)] // TODO handle wrong params
fn read_multi_text(
    current: &EventCommand,
    iter: &mut std::iter::Peekable<impl Iterator<Item = (usize, EventCommand)>>,
    continue_id: u16,
) -> String {
    use std::fmt::Write;

    // RPG maker encodes this in a strange way, its split over multiple commands
    let mut text = string_parameter_at(&current.parameters, 0)
        .unwrap()
        .to_owned();
    // iterate until the next id no longer matches cont_id
    while let Some((_, next)) = iter.next_if(|(_, next)| next.code == continue_id) {
        let continued = string_parameter_at(&next.parameters, 0).unwrap();
        write!(text, " {continued}").unwrap();
    }
    text
}

#[allow(clippy::unwrap_used)] // TODO handle wrong params
fn process_eventcommands(
    commands: Vec<EventCommand>,
    strings: &mut Vec<GameString>,
    context: &str,
) {
    let mut iter = commands.into_iter().enumerate().peekable();
    while let Some((i, next)) = iter.next() {
        match next.code {
            // Show text
            101 => {
                let text = read_multi_text(&next, &mut iter, 401);
                let location = format!("{context}:cmd:{i}:text");
                strings.push(GameString { location, text });
            }
            // Show choices
            102 => {
                for (choice, text) in array_parameter_at(&next.parameters, 0)
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    let text = text.as_str().unwrap().to_owned();
                    let location = format!("{context}:cmd:{i}:choice:{choice}");
                    strings.push(GameString { location, text });
                }
            }
            // Comment
            108 => {
                let text = read_multi_text(&next, &mut iter, 408);
                let location = format!("{context}:cmd:{i}:comment");
                strings.push(GameString { location, text });
            }
            // Conditional branch (may contain script)
            111 => {
                if int_parameter_at(&next.parameters, 0).is_some_and(|v| v == 12) {
                    let text = string_parameter_at(&next.parameters, 1).unwrap().to_owned();
                    let context = format!("{context}:cmd:{i}:branch:script");
                    super::process_script_text(text, strings, context);
                }
            }
            // Label
            118 => {
                let text = string_parameter_at(&next.parameters, 0).unwrap().to_owned();
                let location = format!("{context}:cmd:{i}:label");
                strings.push(GameString { location, text });
            }
            // Goto Label
            119 => {
                let text = string_parameter_at(&next.parameters, 0).unwrap().to_owned();
                let location = format!("{context}:cmd:{i}:goto label");
                strings.push(GameString { location, text });
            }
            // MoveRoute
            209 => {
                let route = route_parameter_at(&next.parameters, 1).unwrap();
                let context = format!("{context}:cmd:{i}:route");
                process_move_route(route, strings, &context);
            }
            // Actor Name
            320 => {
                let text = string_parameter_at(&next.parameters, 1).unwrap().to_owned();
                let location = format!("{context}:cmd:{i}:actor name");
                strings.push(GameString { location, text });
            }
            // Script
            355 => {
                let text = read_multi_text(&next, &mut iter, 655);
                let context = format!("{context}:cmd:{i}:script");
                super::process_script_text(text, strings, context);
            }
            _ => {}
        }
    }
}

#[allow(clippy::unwrap_used)] // TODO handle wrong params
fn process_move_route(route: &MoveRoute, strings: &mut Vec<GameString>, context: &str) {
    for (i, cmd) in route.list.iter().enumerate() {
        if cmd.code == 45 {
            let text = string_parameter_at(&cmd.parameters, 0).unwrap().to_owned();
            let context = format!("{context}:movecmd:{i}:script");
            super::process_script_text(text, strings, context);
        }
    }
}
