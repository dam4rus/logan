use clap::{Arg, App, SubCommand};
use std::{
    path::PathBuf,
    fs::File,
    io::{BufRead, BufReader},
};
use config::Config;
use processors::ColorizeLines;

mod processors;
mod config;

fn main() {
    let matches = App::new("logan")
        .version("0.1")
        .author("Robert Kalmar <rfrostkalmar@gmail.com>")
        .about("Log analyzer CLI application")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .required(true)
            .takes_value(true)
        )
        .arg(Arg::with_name("INPUT")
            .required(true)
            .index(1)
        )
        .get_matches();

    let config_path = PathBuf::from(matches.value_of("config").unwrap());
    let input_path = PathBuf::from(matches.value_of("INPUT").unwrap());

    let config = Config::from_json_file(config_path).unwrap();

    let input_file = File::open(input_path).unwrap();
    let reader = BufReader::new(input_file);

    for line in ColorizeLines::new(config.pattern_colors.as_ref().cloned().unwrap(), reader.lines()) {
        println!("{}", line);
    }
}
