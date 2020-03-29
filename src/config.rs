use crate::processors::{EventPatterns, PatternColor, PatternColors, StateProcessor};
use ansi_term::Color;
use regex::Regex;
use serde_json;
use std::{fs::File, io::BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Config {
    pub pattern_colors: PatternColors,
    pub events: Vec<EventPatterns>,
    pub states: Vec<StateProcessor>,
}

impl Config {
    pub fn from_json_file(file: File) -> Result<Self> {
        Self::from_json_value(serde_json::from_reader(BufReader::new(file))?)
    }

    #[cfg(test)]
    pub fn from_json_str<T: AsRef<str>>(json_str: T) -> Result<Self> {
        Self::from_json_value(serde_json::from_str(json_str.as_ref())?)
    }

    fn from_json_value(json_value: serde_json::Value) -> Result<Self> {
        use serde_json::Value;

        let prefix = match &json_value["prefix"] {
            Value::String(prefix) => Some(prefix),
            Value::Null => None,
            _ => panic!("Invalid config json: unknown prefix value"),
        };

        let pattern_colors = match &json_value["pattern_colors"] {
            Value::Array(pattern_colors) => PatternColors(
                pattern_colors
                    .iter()
                    .map(|pattern_color| {
                        let regex = match &pattern_color["pattern"] {
                            Value::String(pattern) => {
                                create_regex_with_prefix(&prefix.map(|prefix| prefix.as_str()), pattern)?
                            }
                            _ => panic!("Invalid config json: unknown pattern value"),
                        };
                        let color = match &pattern_color["color"] {
                            Value::String(fixed_color) => Color::Fixed(fixed_color.parse()?),
                            _ => panic!("Invalid config json: unknown color value"),
                        };
                        Ok(PatternColor { regex, color })
                    })
                    .collect::<Result<Vec<_>>>()?,
            ),
            Value::Null => Default::default(),
            _ => panic!("Invalid config json: unknown pattern_colors value"),
        };

        let events = match &json_value["event_patterns"] {
            Value::Array(event_patterns) => event_patterns
                .iter()
                .map(|event_pattern| {
                    let start_regex = match &event_pattern["start_pattern"] {
                        Value::String(pattern) => {
                            create_regex_with_prefix(&prefix.map(|prefix| prefix.as_str()), pattern)?
                        }
                        _ => panic!("Invalid config json: unknown start_pattern value"),
                    };
                    let end_regex = match &event_pattern["end_pattern"] {
                        Value::String(pattern) => {
                            create_regex_with_prefix(&prefix.map(|prefix| prefix.as_str()), pattern)?
                        }
                        _ => panic!("Invalid config json: unknown end_pattern value"),
                    };
                    let color = match &event_pattern["color"] {
                        Value::String(fixed_color) => Some(Color::Fixed(fixed_color.parse()?)),
                        Value::Null => None,
                        _ => panic!("Invalid config json: unknown color value"),
                    };
                    Ok(EventPatterns {
                        start_regex,
                        end_regex,
                        color,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            Value::Null => Default::default(),
            _ => panic!("Invalid config json: unknown event_patterns value"),
        };

        let states = match &json_value["state_patterns"] {
            Value::Array(state_patterns) => state_patterns
                .iter()
                .map(|state_pattern| {
                    let regex = match &state_pattern["pattern"] {
                        Value::String(pattern) => {
                            create_regex_with_prefix(&prefix.map(|prefix| prefix.as_str()), pattern)?
                        }
                        _ => panic!("Invalid config json: unknown pattern value"),
                    };
                    let color = match &state_pattern["color"] {
                        Value::String(fixed_color) => Some(Color::Fixed(fixed_color.parse()?)),
                        Value::Null => None,
                        _ => panic!("Invalid config json: unknown color value"),
                    };
                    Ok(StateProcessor::new(regex, color))
                })
                .collect::<Result<Vec<_>>>()?,
            Value::Null => Default::default(),
            _ => panic!("Invalid config json: unknown state_patterns value"),
        };

        Ok(Self {
            pattern_colors,
            events,
            states,
        })
    }
}

pub(crate) fn create_regex_with_prefix(
    prefix: &Option<&str>,
    pattern: &str,
) -> std::result::Result<Regex, regex::Error> {
    Regex::new(format!("{}{}", prefix.map(|prefix| prefix).unwrap_or_default(), pattern).as_str())
}

#[cfg(test)]
mod tests {
    use super::{Config, PatternColors};
    use ansi_term::Color;

    #[test]
    pub fn test_config_from_json() {
        let prefix = r#"[\w]{3} [\d]{2} [\d]{2}:[\d]{2}:[\d]{2} [^ ]* "#;
        let json = r#"{
            "prefix": "[\\w]{3} [\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2} [^ ]* ",
            "pattern_colors": [
                { "pattern": "NetworkManager ", "color": "28" }
            ],
            "event_patterns": [
                {
                    "start_pattern": "Starting Network Manager ",
                    "end_pattern": "Started Network Manager ",
                    "color": "28"
                }
            ],
            "state_patterns": [
                { "pattern": "Switched to [^ ]+", "group": 1, "color": "28" }
            ]
        }"#;

        let config = Config::from_json_str(json).unwrap();
        let PatternColors(pattern_colors) = config.pattern_colors;
        let pattern = &pattern_colors[0];
        assert_eq!(pattern.regex.as_str(), format!(r#"{}NetworkManager "#, prefix));
        assert_eq!(pattern.color, Color::Fixed(28));

        let events = config.events;
        assert_eq!(
            events[0].start_regex.as_str(),
            format!(r#"{}Starting Network Manager "#, prefix)
        );
        assert_eq!(
            events[0].end_regex.as_str(),
            format!(r#"{}Started Network Manager "#, prefix)
        );
        assert_eq!(events[0].color, Some(Color::Fixed(28)));

        let states = config.states;
        assert_eq!(states[0].regex.as_str(), format!(r#"{}Switched to [^ ]+"#, prefix));
        assert_eq!(states[0].color, Some(Color::Fixed(28)));
    }

    #[test]
    pub fn test_empty_config_file() {
        let json = r#"{}"#;

        let config = Config::from_json_str(json).unwrap();
        assert!(config.pattern_colors.0.is_empty());
        assert!(config.events.is_empty());
    }
}
