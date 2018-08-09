use clap::{App, load_yaml, ArgMatches};
use super::conf::Configuration;

fn parse() -> Configuration {
    let yaml = load_yaml!("cli_args.yml")
    let matches = App::from_yaml(yaml).get_matches();
}

