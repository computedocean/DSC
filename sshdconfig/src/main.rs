use args::*;
use atty::Stream;
use clap::Parser;
use config::*;
use config::config::SshdConfig;
use input_helper::*;
use sshdconfig_error::*;
use std::{io::{self, Read}, process::exit};

pub mod args;
pub mod config;
pub mod input_helper;
pub mod sshdconfig_error;

fn main() {
    let args = Cli::parse();

    let stdin: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        Some(input)
    };

    let input_data;
    let curr_sshdconfig;
    match args.command {
        Commands::Get { input_config_path, input_config_json, curr_config_path } => {
            (input_data, curr_sshdconfig) = initial_setup(&input_config_path, 
                &input_config_json, &stdin, &curr_config_path);
            let keywords = match input_data {
                InputData::Text(data) => {
                    match curr_sshdconfig.get_keywords_from_file(&data) {
                        Ok(result) => Some(result),
                        Err(e) => {
                            eprintln!("Invalid input error: {}", e);
                            exit(EXIT_INPUT_INVALID);
                        }
                    }
                }
                InputData::Json(data) => {
                    match curr_sshdconfig.get_keywords_from_json(&data) {
                        Ok(result) => Some(result),
                        Err(e) => {
                            eprintln!("Invalid input error: {}", e);
                            exit(EXIT_INPUT_INVALID);
                        }
                    }
                }
                InputData::None => {
                    None
                }
            };
            match curr_sshdconfig.get(&keywords) {
                Ok(result) => {
                    println!("{}", result);
                },
                Err(e) => {
                    eprintln!("Error getting sshd config: {}", e);
                    exit(EXIT_UNSPECIFIED_ERR);
                }
            }
        }
        Commands::Set { input_config_path, input_config_json, curr_config_path } => {
            (input_data, curr_sshdconfig) = initial_setup(&input_config_path, 
                &input_config_json, &stdin, &curr_config_path);
            let new_sshdconfig = initialize_new_config(&input_data);
            match curr_sshdconfig.set(&new_sshdconfig) {
                Ok(result) => {
                    if !result {
                        exit(EXIT_NOT_IN_DESIRED_STATE);
                    }
                },
                Err(e) => {
                    eprintln!("Error setting sshd config: {}", e);
                    exit(EXIT_UNSPECIFIED_ERR);
                }
            }
        }
        Commands::Test { input_config_path, input_config_json, curr_config_path } => {
            (input_data, curr_sshdconfig) = initial_setup(&input_config_path, 
                &input_config_json, &stdin, &curr_config_path);
            let new_sshdconfig = initialize_new_config(&input_data);
            match curr_sshdconfig.test(&new_sshdconfig) {
                Ok(result) => {
                    println!("{}", result.0);
                    if !result.1 {
                        exit(EXIT_NOT_IN_DESIRED_STATE);
                    }
                },
                Err(e) => {
                    eprintln!("Error testing sshd config: {}", e);
                    exit(EXIT_UNSPECIFIED_ERR);
                }
            }
        }
    }
    exit(EXIT_SUCCESS);
}

// mainly an example at this point
#[test]
fn test_config() {
    let input_json: &str = r#"
    {
        "passwordauthentication": "Yes",
        "syslogfacility": "INFO",
        "subsystem": [
            {
                "name": "powershell",
                "value": "pwsh.exe"
            }
        ],
        "port": [
            { "value": "24" },
            { "value": "23" }
        ],
        "match": {
            "group": [
                {
                    "criteria": "administrators",
                    "passwordauthentication": "Yes",
                    "_ensure": "Present"
                }
            ]
        }
    }
    "#;
    let config: SshdConfig = serde_json::from_str(input_json).unwrap();
    let json = config.to_json();
}
