use ansi_term::Color;
use clap::{App, Arg, SubCommand};
use config::{create_regex_with_prefix, Config};
use processors::{Colorize, EventPatterns, EventProcessor, PatternColor, PatternColors, Processor, StateProcessor};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

mod config;
mod processors;

fn main() {
    let matches = App::new("logan")
        .version("0.1")
        .author("Róbert Kalmár <rfrostkalmar@gmail.com>")
        .about("Log analyzer CLI application")
        .subcommand(SubCommand::with_name("use-config").arg(Arg::with_name("config_path").required(true)))
        .subcommand(
            SubCommand::with_name("colorize")
                .arg(Arg::with_name("prefix").short("P").long("prefix").takes_value(true))
                .arg(
                    Arg::with_name("patterns")
                        .short("p")
                        .long("pattern")
                        .multiple(true)
                        .number_of_values(2)
                        .required(true)
                        .value_names(&["COLOR", "PATTERN"]),
                ),
        )
        .subcommand(
            SubCommand::with_name("events")
                .arg(Arg::with_name("prefix").short("P").long("prefix").takes_value(true))
                .arg(Arg::with_name("color").short("c").long("color").takes_value(true))
                .arg(Arg::with_name("start").required(true))
                .arg(Arg::with_name("end").required(true)),
        )
        .subcommand(
            SubCommand::with_name("states")
                .arg(Arg::with_name("prefix").short("P").long("prefix").takes_value(true))
                .arg(Arg::with_name("color").short("c").long("color").takes_value(true))
                .arg(Arg::with_name("regex").required(true)),
        )
        .arg(Arg::with_name("INPUT").required(true))
        .get_matches();

    let input_path = PathBuf::from(matches.value_of("INPUT").unwrap());

    let mut processors = match matches.subcommand() {
        ("use-config", Some(config_matches)) => {
            let config_path = PathBuf::from(config_matches.value_of("config_path").unwrap());
            let config_file = File::open(config_path).expect("Failed to open config file");
            let config = Config::from_json_file(config_file).unwrap();
            config
                .events
                .iter()
                .map(|event| Box::new(EventProcessor::new(event.clone())) as Box<dyn Processor>)
                .chain(
                    config
                        .states
                        .iter()
                        .map(|state| Box::new(state.clone()) as Box<dyn Processor>),
                )
                .collect::<Vec<_>>()
        }
        ("colorize", Some(colorize_matches)) => {
            let prefix = colorize_matches.value_of("prefix");

            let pattern_colors = colorize_matches
                .values_of("patterns")
                .unwrap()
                .collect::<Vec<_>>()
                .as_slice()
                .chunks_exact(2)
                .map(|params| {
                    let color = Color::Fixed(params[0].parse::<u8>()?);
                    let regex = create_regex_with_prefix(&prefix, params[1])?;
                    Ok(PatternColor { color, regex })
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()
                .unwrap();

            vec![Box::new(Colorize::new(PatternColors(pattern_colors))) as Box<dyn Processor>]
        }
        ("events", Some(events_matches)) => {
            let prefix = events_matches.value_of("prefix");

            let color = match events_matches.value_of("color").map(|color| color.parse::<u8>()) {
                Some(Ok(color)) => Some(Color::Fixed(color)),
                Some(Err(err)) => panic!("Invalid color value ({})", err),
                None => None,
            };

            let start_regex = create_regex_with_prefix(&prefix, events_matches.value_of("start").unwrap())
                .expect("Invalid start regex pattern");

            let end_regex = create_regex_with_prefix(&prefix, events_matches.value_of("end").unwrap())
                .expect("Invalid end regex pattern");

            vec![Box::new(EventProcessor::new(EventPatterns {
                start_regex,
                end_regex,
                color,
            })) as Box<dyn Processor>]
        }
        ("states", Some(states_matches)) => {
            let prefix = states_matches.value_of("prefix");

            let color = match states_matches.value_of("color").map(|color| color.parse::<u8>()) {
                Some(Ok(color)) => Some(Color::Fixed(color)),
                Some(Err(err)) => panic!("Invalid color value ({})", err),
                None => None,
            };

            let regex = create_regex_with_prefix(&prefix, states_matches.value_of("regex").unwrap())
                .expect("Invalid regex pattern");

            vec![Box::new(StateProcessor::new(regex, color)) as Box<dyn Processor>]
        }
        _ => panic!("Invalid command line arguments"),
    };

    let input_file = File::open(input_path).expect("Failed to open input file");
    let reader = BufReader::new(input_file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line from input file");
        for processor in &mut processors {
            if let Some(output) = processor.process_line(line.as_str()) {
                println!("{}", output);
            }
        }
    }
}
