use crate::modrinth::{ModVersionFile, MpmMod};
use crate::polymc::PolyMC;
use crate::{modrinth::ModVersion, PolyInstance};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::ErrorKind,
};

pub struct ModpmLockfile {}

#[derive(Serialize, Deserialize, Debug)]
pub struct LockfileMod {
    pub version: ModVersion,
    pub file: ModVersionFile,
    pub mpm_mod: Option<MpmMod>,
}

impl ModpmLockfile {
    pub fn add_to_lockfile(instance: PolyInstance, version: &ModVersion, file: &ModVersionFile) {
        let mut current_lockfile = ModpmLockfile::get_lockfile(instance.clone());

        current_lockfile.push(LockfileMod {
            version: version.clone(),
            file: file.clone(),
            mpm_mod: None,
        });

        let new_lockfile_string =
            json5::to_string(&current_lockfile).expect("Couldn't serialize a lockfile");

        let result = fs::write(
            format!(
                "{}/instances/{}/.minecraft/mods/.modpm_lockfile.json",
                PolyMC::get_directory(),
                instance.folder_name
            ),
            &new_lockfile_string,
        );

        match result {
            Ok(_) => {}
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    File::create(format!(
                        "{}/instances/{}/.minecraft/mods/.modpm_lockfile.json",
                        PolyMC::get_directory(),
                        instance.folder_name
                    ))
                    .expect("Couldn't create a lockfile");
                    fs::write(
                        format!(
                            "{}/{}/.minecraft/mods/.modpm_lockfile.json",
                            PolyMC::get_directory(),
                            instance.folder_name
                        ),
                        new_lockfile_string,
                    )
                    .expect("something went really really wrong while making a lockfile");
                } else {
                    panic!("something went really really wrong while making a lockfile")
                };
            }
        }
    }

    pub fn get_lockfile(instance: PolyInstance) -> Vec<LockfileMod> {
        let mut current_lockfile_string = "".to_string();
        let possible_lockfile_string = &fs::read_to_string(format!(
            "{}/instances/{}/.minecraft/mods/.modpm_lockfile.json",
            PolyMC::get_directory(),
            instance.folder_name
        ));
        match possible_lockfile_string {
            Ok(_) => {
                possible_lockfile_string.as_ref().expect("wtf")[..]
                    .clone_into(&mut current_lockfile_string);
            }
            Err(_) => {
                "[]".clone_into(&mut current_lockfile_string);
            }
        }

        let current_lockfile: Vec<LockfileMod> =
            json5::from_str(&current_lockfile_string[..]).expect("Couldn't deserialize a lockfile");

        current_lockfile
    }
}
