use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use toml::{map::Map, Value};

use crate::config::schema::{Config, ConfigAsset};
use crate::model::asset_name::AssetName;

#[derive(Debug, PartialEq, Eq)]
pub enum TomlError {
    IO(String),
    Parse(toml::de::Error),
    Decode,
}

impl TomlError {
    pub fn display(&self) -> String {
        match self {
            TomlError::IO(e) => format!("[IO Error] {}", e),
            TomlError::Parse(e) => format!("[Parsing Error] {}", e),
            TomlError::Decode => "[Decode Error]".to_string(),
        }
    }
}

pub fn parse_file(config_path: &PathBuf) -> Result<Config, TomlError> {
    let contents = fs::read_to_string(config_path).map_err(|e| TomlError::IO(format!("{}", e)))?;

    parse_string(&contents)
}

fn parse_string(contents: &str) -> Result<Config, TomlError> {
    contents
        .parse::<Value>()
        .map_err(TomlError::Parse)
        .and_then(|toml| match decode_config(toml) {
            None => Err(TomlError::Decode),
            Some(config) => Ok(config),
        })
}

fn decode_config(toml: Value) -> Option<Config> {
    let str_store_directory = toml.get("store_directory")?.as_str()?;
    let store_directory = String::from(str_store_directory);

    let mut tools = BTreeMap::new();

    for (key, val) in toml.as_table()?.iter() {
        if let Value::Table(table) = val {
            tools.insert(key.clone(), decode_config_asset(table));
        }
    }

    Some(Config {
        store_directory,
        tools,
    })
}

fn decode_config_asset(table: &Map<String, Value>) -> ConfigAsset {
    let owner = str_by_key(table, "owner");
    let repo = str_by_key(table, "repo");
    let exe_name = str_by_key(table, "exe_name");
    let asset_name = decode_asset_name(table);
    let tag = str_by_key(table, "tag");

    ConfigAsset {
        owner,
        repo,
        exe_name,
        asset_name,
        tag,
    }
}

fn decode_asset_name(table: &Map<String, Value>) -> AssetName {
    match table.get("asset_name").and_then(|t| t.as_table()) {
        None => AssetName {
            linux: None,
            macos: None,
            windows: None,
        },

        Some(table) => {
            let linux = str_by_key(table, "linux");
            let macos = str_by_key(table, "macos");
            let windows = str_by_key(table, "windows");

            AssetName {
                linux,
                macos,
                windows,
            }
        }
    }
}

fn str_by_key(table: &Map<String, Value>, key: &str) -> Option<String> {
    table.get(key).and_then(|v| v.as_str()).map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file() {
        let toml = "";
        let res = parse_string(toml);

        assert_eq!(res, Err(TomlError::Decode));
    }

    #[test]
    fn store_directory_is_dotted() {
        let toml = "store.directory = \"pancake\"";
        let res = parse_string(toml);

        assert_eq!(res, Err(TomlError::Decode));
    }

    #[test]
    fn store_directory_is_a_number() {
        let toml = "store_directory = 42";
        let res = parse_string(toml);

        assert_eq!(res, Err(TomlError::Decode));
    }

    #[test]
    fn only_store_directory() {
        let toml = "store_directory = \"pancake\"";
        let res = parse_string(toml);

        let cfg = Config {
            store_directory: String::from("pancake"),
            tools: BTreeMap::new(),
        };

        assert_eq!(res, Ok(cfg));
    }

    #[test]
    fn single_empty_tool() {
        let toml = r#"
            store_directory = "pancake"

            [ripgrep]
        "#;

        let res = parse_string(toml);

        let cfg = Config {
            store_directory: String::from("pancake"),
            tools: BTreeMap::from([(
                "ripgrep".to_owned(),
                ConfigAsset {
                    owner: None,
                    repo: None,
                    exe_name: None,
                    asset_name: AssetName {
                        linux: None,
                        macos: None,
                        windows: None,
                    },
                    tag: None,
                },
            )]),
        };

        assert_eq!(res, Ok(cfg));
    }

    #[test]
    fn two_empty_tools() {
        let toml = r#"
            store_directory = "pancake"

            [ripgrep]
            [bat]
        "#;

        let res = parse_string(toml);

        let cfg = Config {
            store_directory: String::from("pancake"),
            tools: BTreeMap::from([
                (
                    "ripgrep".to_owned(),
                    ConfigAsset {
                        owner: None,
                        repo: None,
                        exe_name: None,
                        asset_name: AssetName {
                            linux: None,
                            macos: None,
                            windows: None,
                        },
                        tag: None,
                    },
                ),
                (
                    "bat".to_owned(),
                    ConfigAsset {
                        owner: None,
                        repo: None,
                        exe_name: None,
                        asset_name: AssetName {
                            linux: None,
                            macos: None,
                            windows: None,
                        },
                        tag: None,
                    },
                ),
            ]),
        };

        assert_eq!(res, Ok(cfg));
    }

    #[test]
    fn single_partial_tool() {
        let toml = r#"
            store_directory = "pancake"

            [ripgrep]
            owner = "me"
            asset_name.linux = "R2D2"
        "#;

        let res = parse_string(toml);

        let cfg = Config {
            store_directory: String::from("pancake"),
            tools: BTreeMap::from([(
                "ripgrep".to_owned(),
                ConfigAsset {
                    owner: Some("me".to_owned()),
                    repo: None,
                    exe_name: None,
                    asset_name: AssetName {
                        linux: Some("R2D2".to_owned()),
                        macos: None,
                        windows: None,
                    },
                    tag: None,
                },
            )]),
        };

        assert_eq!(res, Ok(cfg));
    }

    #[test]
    fn single_full_tool() {
        let toml = r#"
            store_directory = "pancake"

            [ripgrep]
            owner = "me"
            repo = "some_repo"
            exe_name = "rg"
            asset_name.linux = "R2D2"
            asset_name.macos = "C3-PO"
            asset_name.windows = "IG-88"
            tag = "4.2.0"
        "#;

        let res = parse_string(toml);

        let cfg = Config {
            store_directory: String::from("pancake"),
            tools: BTreeMap::from([(
                "ripgrep".to_owned(),
                ConfigAsset {
                    owner: Some("me".to_owned()),
                    repo: Some("some_repo".to_owned()),
                    exe_name: Some("rg".to_owned()),
                    asset_name: AssetName {
                        linux: Some("R2D2".to_owned()),
                        macos: Some("C3-PO".to_owned()),
                        windows: Some("IG-88".to_owned()),
                    },
                    tag: Some("4.2.0".to_owned()),
                },
            )]),
        };

        assert_eq!(res, Ok(cfg));
    }
}
