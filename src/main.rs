use clap::{Arg, App};
use std::{
    path::PathBuf,
    fs::File,
    io::{BufRead, BufReader},
};
use config::Config;
use processors::{EventProcessor, Processor};

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
    let mut processors: Vec<Box<dyn Processor>> = Vec::new();
    processors.extend(
        config
            .events
            .iter()
            .map(|event| Box::new(EventProcessor::new(event.clone())) as Box<dyn Processor>)
            .collect::<Vec<_>>()
    );
    processors.extend(
        config.states.iter().map(|state| Box::new(state.clone()) as Box<dyn Processor>).collect::<Vec<_>>()
    );

    let input_file = File::open(input_path).unwrap();
    let reader = BufReader::new(input_file);
    for line in reader.lines() {
        let line = line.unwrap();
        for processor in &mut processors {
            if let Some(output) = processor.process_line(line.as_str()) {
                println!("{}", output);
            }
        }
    }
}
