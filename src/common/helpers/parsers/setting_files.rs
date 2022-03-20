use crate::common::constants::file_paths::FilePaths;
use crate::common::models::secrets::SecretsSettings;
use std::fs::File;

use crate::common::models::config::ConfigSettings;

pub struct SettingFiles {
    pub config: ConfigSettings,
    pub secrets: SecretsSettings,
}

impl<'a> SettingFiles {
    fn config_file() -> ConfigSettings {
        let file_path = FilePaths::CONFIG;
        let f = File::open(file_path);
        let f_ok = match f {
            Ok(f) => f,
            Err(e) => {
                paniq!(
                    "An error occurred while reading the '{}' file (P00002): {:?}",
                    file_path,
                    e
                );
            }
        };

        let data: Result<ConfigSettings, serde_yaml::Error> = serde_yaml::from_reader(f_ok);

        match data {
            Ok(d) => d,
            Err(e) => {
                paniq!(
                    "An error occurred while deserializing the '{}' file (P00003): {:?}",
                    file_path,
                    e
                );
            }
        }
    }

    fn secrets_file() -> SecretsSettings {
        let file_path = FilePaths::SECRETS;
        let f = File::open(file_path);

        let f_ok = match f {
            Ok(f) => f,
            Err(e) => {
                paniq!(
                    "An error occurred while reading the '{}' file (P00004): {:?}",
                    file_path,
                    e
                );
            }
        };

        let data: Result<SecretsSettings, serde_yaml::Error> = serde_yaml::from_reader(f_ok);

        match data {
            Ok(d) => d,
            Err(e) => {
                paniq!(
                    "An error occurred while deserializing the '{}' file (P00005): {:?}",
                    file_path,
                    e
                );
            }
        }
    }

    pub fn new() -> SettingFiles {
        log::debug!("reading the config files...");

        let c = SettingFiles::config_file();
        let s = SettingFiles::secrets_file();

        SettingFiles {
            config: c,
            secrets: s,
        }
    }
}
