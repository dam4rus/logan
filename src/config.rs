use serde_json;
use std::{
    fs::File,
    io::BufReader,
    path::Path,
};
use regex::Regex;
use ansi_term::Color;
use crate::processors::StateProcessor;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct PatternColor {
    pub regex: Regex,
    pub color: Color,
}

#[derive(Debug, Clone, Default)]
pub struct PatternColors(pub Vec<PatternColor>);

impl PatternColors {
    pub fn find_color_for_pattern<T: AsRef<str>>(&self, line: T) -> Option<Color> {
        self.0.iter().find(|pattern| pattern.regex.is_match(line.as_ref())).map(|pattern| pattern.color)
    }
}

#[derive(Debug, Clone)]
pub struct EventPatterns {
    pub start_regex: Regex,
    pub end_regex: Regex,
    pub color: Option<Color>,
}

#[derive(Debug)]
pub struct Config {
    pub pattern_colors: PatternColors,
    pub events: Vec<EventPatterns>,
    pub states: Vec<StateProcessor>,
}

impl Config {
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Self::from_json_value(serde_json::from_reader(reader)?)
    }

    pub fn from_json_str<T: AsRef<str>>(json_str: T) -> Result<Self> {
        Self::from_json_value(serde_json::from_str(json_str.as_ref())?)
    }

    fn from_json_value(json_value: serde_json::Value) -> Result<Self> {
        use serde_json::Value;

        let pattern_colors = match &json_value["pattern_colors"] {
            Value::Array(pattern_colors) => {
                PatternColors(pattern_colors
                    .iter()
                    .map(|pattern_color| {
                        let regex = match &pattern_color["pattern"] {
                            Value::String(pattern) => Regex::new(pattern.as_str())?,
                            _ => panic!("Invalid config json: unknown pattern value"),
                        };
                        let color = match &pattern_color["color"] {
                            Value::String(fixed_color) => Color::Fixed(fixed_color.parse()?),
                            _ => panic!("Invalid config json: unknown color value"),
                        };
                        Ok(PatternColor {
                            regex,
                            color,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
                )
            }
            Value::Null => Default::default(),
            _ => panic!("Invalid config json: unknown pattern_colors value"),
        };

        let events = match &json_value["event_patterns"] {
            Value::Array(event_patterns) => {
                event_patterns
                    .iter()
                    .map(|event_pattern| {
                        let start_regex = match &event_pattern["start_pattern"] {
                            Value::String(pattern) => Regex::new(pattern.as_str())?,
                            _ => panic!("Invalid config json: unknown start_pattern value"),
                        };
                        let end_regex = match &event_pattern["end_pattern"] {
                            Value::String(pattern) => Regex::new(pattern.as_str())?,
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
                    .collect::<Result<Vec<_>>>()?
            }
            Value::Null => Default::default(),
            _ => panic!("Invalid config json: unknown event_patterns value"),
        };

        let states = match &json_value["state_patterns"] {
            Value::Array(state_patterns) => {
                state_patterns
                    .iter()
                    .map(|state_pattern| {
                        let regex = match &state_pattern["pattern"] {
                            Value::String(pattern) => Regex::new(pattern.as_str())?,
                            _ => panic!("Invalid config json: unknown pattern value"),
                        };
                        let capture_group = match &state_pattern["group"] {
                            Value::Number(group) if group.is_u64() => group.as_u64().unwrap() as usize,
                            _ => panic!("Invalid config json: unknown group value"),
                        };
                        let color = match &state_pattern["color"] {
                            Value::String(fixed_color) => Some(Color::Fixed(fixed_color.parse()?)),
                            Value::Null => None,
                            _ => panic!("Invalid config json: unknown color value"),
                        };
                        Ok(StateProcessor::new(regex, capture_group, color))
                    })
                    .collect::<Result<Vec<_>>>()?
            }
            Value::Null => Default::default(),
            _ => panic!("Invalid config json: unknown state_patterns value"),
        };

        Ok(Self { pattern_colors, events, states })
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, PatternColors};
    use ansi_term::Color;

    #[test]
    pub fn test_config_from_json() {
        let prefix_regex = r#"[\\w]{3} [\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2} [^ ]* "#;
        let prefix = r#"[\w]{3} [\d]{2} [\d]{2}:[\d]{2}:[\d]{2} [^ ]* "#;
        let json = format!(r#"{{
            "pattern_colors": [
                {{ "pattern": "{prefix}NetworkManager ", "color": "28" }}
            ],
            "event_patterns": [
                {{
                    "start_pattern": "{prefix}Starting Network Manager ",
                    "end_pattern": "{prefix}Started Network Manager ",
                    "color": "28"
                }}
            ],
            "state_patterns": [
                {{ "pattern": "{prefix}Switched to ([^ ]+)", "group": 1, "color": "28" }}
            ]
        }}"#,
            prefix=prefix_regex,
        );

        let config = Config::from_json_str(json).unwrap();
        let PatternColors(pattern_colors) = config.pattern_colors;
        let pattern = &pattern_colors[0];
        assert_eq!(pattern.regex.as_str(), format!(r#"{}NetworkManager "#, prefix));
        assert_eq!(pattern.color, Color::Fixed(28));

        let events = config.events;
        assert_eq!(events[0].start_regex.as_str(), format!(r#"{}Starting Network Manager "#, prefix));
        assert_eq!(events[0].end_regex.as_str(), format!(r#"{}Started Network Manager "#, prefix));
        assert_eq!(events[0].color, Some(Color::Fixed(28)));

        let states = config.states;
        assert_eq!(states[0].regex.as_str(), format!(r#"{}Switched to ([^ ]+)"#, prefix));
        assert_eq!(states[0].capture_group, 1);
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