use ansi_term::Color;
use clap::{App, Arg, SubCommand, ArgMatches};
use config::{create_regex_with_prefix, Config};
use processors::{Colorize, EventPatterns, EventProcessor, PatternColor, Processor, StateProcessor};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};
use crate::error::ParseColorError;

mod config;
mod error;
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
                        .value_names(&["PATTERN", "COLOR"]),
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

    let mut processors = match parse_processors(matches) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let mut last_process_required_separator = false;
    let mut has_output = false;
    let input_file = File::open(input_path).expect("Failed to open input file");
    let reader = BufReader::new(input_file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line from input file");
        for processor in &mut processors {
            if let Some(output) = processor.process_line(line.as_str()) {
                if has_output && (last_process_required_separator || processor.requires_separator()) {
                    println!("{sep}\n{}", output, sep="-".repeat(50));
                } else {
                    println!("{}", output);
                }

                has_output = true;
                last_process_required_separator = processor.requires_separator();
            }
        }
    }

    println!("");
    for processor in &processors {
        if let Some(result) = processor.result() {
            println!("{}", result);
        }
    }
}

fn parse_processors(matches: ArgMatches) -> Result<Vec<Box<dyn Processor>>, Box<dyn std::error::Error>> {
    match matches.subcommand() {
        ("use-config", Some(config_matches)) => {
            let config_path = PathBuf::from(config_matches.value_of("config_path").unwrap());
            let config_file = File::open(config_path).map_err(|err| format!("Failed to open config file: {}", err))?;
            let config = Config::from_json_file(config_file)?;

            let mut processors = config.pattern_colors
                .map(|pattern_colors| vec![Box::new(Colorize::new(pattern_colors)) as Box<dyn Processor>])
                .unwrap_or_default();

            processors.extend(
                config
                    .events
                    .into_iter()
                    .map(|event| Box::new(EventProcessor::new(event)) as Box<dyn Processor>)
                );

            processors.extend(
                config
                    .states
                    .into_iter()
                    .map(|state| Box::new(state) as Box<dyn Processor>),
            );

            Ok(processors)
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
                    let regex_value = params[0];
                    let color_value = params[1];
                    let color = Color::Fixed(color_value.parse::<u8>().map_err(|err| ParseColorError::new(color_value, err))?);
                    let regex = create_regex_with_prefix(&prefix, regex_value)?;
                    Ok(PatternColor { color, regex })
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

            Ok(vec![Box::new(Colorize::new(pattern_colors)) as Box<dyn Processor>])
        }
        ("events", Some(events_matches)) => {
            let prefix = events_matches.value_of("prefix");
            let color = events_matches
                .value_of("color")
                .map(|color| color.parse::<u8>().map_err(|err| ParseColorError::new(color, err)))
                .transpose()?
                .map(|color| Color::Fixed(color));

            let start_regex_value = events_matches.value_of("start").unwrap();
            let start_regex = create_regex_with_prefix(&prefix, start_regex_value)?;

            let end_regex_value = events_matches.value_of("end").unwrap();
            let end_regex = create_regex_with_prefix(&prefix, end_regex_value)?;

            Ok(vec![Box::new(EventProcessor::new(EventPatterns {
                start_regex,
                end_regex,
                color,
            })) as Box<dyn Processor>])
        }
        ("states", Some(states_matches)) => {
            let prefix = states_matches.value_of("prefix");
            let color = states_matches
                .value_of("color")
                .map(|color| color.parse::<u8>().map_err(|err| ParseColorError::new(color, err)))
                .transpose()?
                .map(|color| Color::Fixed(color));

            let regex_value = states_matches.value_of("regex").unwrap();
            let regex = create_regex_with_prefix(&prefix, regex_value)?;

            Ok(vec![Box::new(StateProcessor::new(regex, color)) as Box<dyn Processor>])
        }
        _ => unreachable!(),
    }
}