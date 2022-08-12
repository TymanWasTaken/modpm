use std::collections::HashMap;

use clap::{arg, Command};
use modpm::{ask_user, modrinth::MpmMod, PolyMC};

fn cli() -> Command<'static> {
    Command::new("modpm")
        .about("A Minecraft mod package manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(false)
        .subcommand(
            Command::new("query")
                .about("Queries a mod")
                .arg(arg!(<MOD> "The mod to query."))
                .arg(arg!(-v --versions "Show recent versions.").action(clap::ArgAction::SetTrue))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("download")
                .about("Downloads a mod")
                .arg(arg!(<MOD> "The mod to download."))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("update").about("Update all of your mods from a specific instance"),
        )
    // .subcommand(Command::new("polymc").about("testing lmao"))
    // .subcommand(Command::new("test").about("even more testing"))
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("query", sub_matches)) => {
            let mmod = sub_matches.get_one::<String>("MOD").expect("required");

            let versions = sub_matches.get_one::<bool>("versions").expect("how");

            let mod_data = match MpmMod::new(mmod).await {
                Ok(data) => data,
                Err(_) => MpmMod::new_from_hash(mmod).await,
            };

            println!(
                "I found {}{}, which is licensed under {}, and located at {}",
                ansi_term::Color::Green.paint(&mod_data.title),
                ansi_term::Color::RGB(128, 128, 128).paint(format!(" ({})", mod_data.id)),
                ansi_term::Color::Green.paint(&mod_data.license.name),
                ansi_term::Color::RGB(255, 165, 0).paint(&mod_data.source_url)
            );
            println!("{}", mod_data.description);

            let mut members: HashMap<String, Vec<String>> = HashMap::new();
            members.insert("Owner".to_string(), vec![]);

            for member in mod_data.members {
                let _entry = match members.entry(member.role) {
                    std::collections::hash_map::Entry::Vacant(role) => {
                        let new_value = vec![member.user.name.unwrap_or(member.user.username)];
                        role.insert(new_value);
                    }
                    std::collections::hash_map::Entry::Occupied(mut role) => {
                        role.get_mut()
                            .push(member.user.name.unwrap_or(member.user.username));
                    }
                };
            }

            println!(
                "Owner: {}",
                ansi_term::Color::Purple.paint(members.remove("Owner").unwrap().join(", "))
            );

            for (role, people) in members {
                println!("{}: {}", role, people.join(", "));
            }
            if versions == &true {
                let mut versions_vec = mod_data.versions.clone();
                versions_vec.reverse();

                for v in versions_vec {
                    println!(
                        "{} {}\n\t├ Game versions: {}\n\t└ Modloaders: {}",
                        ansi_term::Color::Green.paint(v.name),
                        ansi_term::Color::RGB(128, 128, 128)
                            .paint(&format!("({})", v.version_number)[..]),
                        ansi_term::Color::RGB(139, 69, 19).paint(v.game_versions.join(", ")),
                        ansi_term::Color::Purple.paint(v.loaders.join(", "))
                    );
                }
            }
        }
        Some(("download", sub_matches)) => {
            let mod_arg = sub_matches.get_one::<String>("MOD").expect("required");
            let mod_data = match MpmMod::new(mod_arg).await {
                Ok(data) => data,
                Err(_) => MpmMod::new_from_hash(mod_arg).await,
            };

            println!(
                "I found {}{} by {} - {}\n",
                ansi_term::Color::Green.paint(&mod_data.title),
                ansi_term::Color::RGB(128, 128, 128).paint(format!(" ({})", mod_data.id)),
                ansi_term::Color::Purple.paint(
                    mod_data
                        .get_owner()
                        .expect("Couldn't get a mod's owner")
                        .user
                        .display_name()
                ),
                mod_data.description
            );

            let instances = PolyMC::get_instances().expect("Couldn't get PolyMC instances.");
            for instance in &instances {
                println!(
                    "{}: {} - {} {}",
                    instance.id,
                    ansi_term::Color::Blue.paint(&instance.name),
                    ansi_term::Color::Purple.paint(&instance.modloader),
                    ansi_term::Color::Green.paint(&instance.game_version)
                );
            }

            let instance_id = ask_user("What instance do you want to download this mod to? ");

            let instance = instances
                .into_iter()
                .find(|i| i.id.to_string() == instance_id)
                .expect("Couldn't find that instance.");

            mod_data.download(instance).await;
        }
        Some(("update", _)) => {
            println!("hi! just putting this in here to remind me to do it later lol, it's really not done yet.
if for whatever reason you've installed this from git, instead of cargo, please yell at me.")
        }
        /*
                Some(("polymc", _)) => {
                    println!("hi yes i literally just use this for testing shit\nthis will be removed before an actual release lmao");
                    let instances = PolyMC::get_instances().unwrap();

                    for instance in instances {
                        println!(
                            "{}: {} - {} {}",
                            instance.id, instance.name, instance.modloader, instance.game_version
                        );
                    }
                }
                Some(("test", _)) => {
                    use chrono::prelude::*;

                    let utc = DateTime::parse_from_rfc3339("2022-06-21T17:11:12+00:00").unwrap();
                    println!("{}", utc.timestamp());
                }
        */
        _ => unreachable!(),
    }
}
