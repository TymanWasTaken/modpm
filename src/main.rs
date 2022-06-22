use clap::{arg, Command};
use modpm::{ask_user, format_to_vec_of_strings, PolyMC};
pub mod data_structs;
use data_structs::Mod;

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
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("download")
                .about("Downloads a mod")
                .arg(arg!(<MOD> "The mod to download."))
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("polymc").about("testing lmao"))
        .subcommand(Command::new("test").about("even more testing"))
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("query", sub_matches)) => {
            let mmod = sub_matches.get_one::<String>("MOD").expect("required");
            let queried_mod = Mod::query(mmod).await.unwrap();

            println!(
                "I found {}, with the ID {}.",
                queried_mod.name, queried_mod.id
            );
        }
        Some(("download", sub_matches)) => {
            let mmod = sub_matches.get_one::<String>("MOD").expect("required");
            let queried_mod = Mod::query(mmod).await.unwrap();

            println!(
                "I found {}, with the ID {}.",
                queried_mod.name, queried_mod.id
            );

            let instances = PolyMC::get_instances().expect("Couldn't get PolyMC instances.");
            for instance in &instances {
                println!(
                    "{}: {} - {} {}",
                    instance.id, instance.name, instance.modloader, instance.game_version
                );
            }

            let instance_id = ask_user("What instance do you want to download this mod to? ");

            let instance = &instances
                .into_iter()
                .find(|i| i.id.to_string() == instance_id)
                .expect("Couldn't find that instance.");

            println!("{:?}", instance);

            queried_mod.download(*instance);
        }
        Some(("polymc", _)) => {
            let instances = PolyMC::get_instances().unwrap();

            for instance in instances {
                println!(
                    "{}: {} - {} {}",
                    instance.id, instance.name, instance.modloader, instance.game_version
                );
            }
        }
        Some(("test", _)) => {
            println!("{}", PolyMC::get_directory());
        }
        _ => unreachable!(),
    }
}
