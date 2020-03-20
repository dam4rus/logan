use crate::processors::{PatternColor, PatternColors};
use serde_json;
use std::{
    fs::File,
    io::BufReader,
    path::Path,
};
use regex::Regex;
use ansi_term::Color;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Config {
    pub pattern_colors: Option<PatternColors>,
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
                Some(PatternColors(pattern_colors
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
                ))
            }
            Value::Null => None,
            _ => panic!("Invalid config json: unknown pattern_colors value"),
        };

        Ok(Self { pattern_colors })
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::processors::PatternColors;
    use ansi_term::Color;

    #[test]
    pub fn test_config_from_json() {
        let json = r#"{
            "pattern_colors": [
                { "pattern": "[\\w]{3} [\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2} [^ ]* NetworkManager ", "color": "28" }
            ]
        }"#;

        let config = Config::from_json_str(json).unwrap();
        let PatternColors(pattern_colors) = config.pattern_colors.unwrap();
        let pattern = &pattern_colors[0];
        assert_eq!(pattern.regex.as_str(), "[\\w]{3} [\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2} [^ ]* NetworkManager ");
        assert_eq!(pattern.color, Color::Fixed(28));
    }

    #[test]
    pub fn test_config_from_json_no_pattern_colors() {
        let json = r#"{}"#;

        let config = Config::from_json_str(json).unwrap();
        assert!(config.pattern_colors.is_none());
    }
}