use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    num::ParseIntError,
};

pub enum ConfigError {
    JsonParse(serde_json::error::Error),
    JsonType(&'static str, JsonType),
    Regex(&'static str, regex::Error),
    ParseInt(&'static str, ParseIntError),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Invalid configuration file: {}",
            match self {
                ConfigError::JsonParse(err) => format!("{}", err),
                ConfigError::JsonType(name, expected_type) => format!(
                    r#"Invalid Json value type for "{}" (Expected: {})"#,
                    name, expected_type
                ),
                ConfigError::Regex(name, err) => format!(r#"Invalid regex for "{}". ({})"#, name, err),
                ConfigError::ParseInt(name, err) => format!(r#"Failed to parse "{}". ({})"#, name, err),
            }
        )
    }
}

impl Debug for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ConfigError({})",
            match self {
                ConfigError::JsonParse(err) => format!("{:?}", err),
                ConfigError::JsonType(name, expected_type) =>
                    format!("name: {:?}, expected_type: {:?}", name, expected_type),
                ConfigError::Regex(name, err) => format!("name: {:?}, err: {:?}", name, err),
                ConfigError::ParseInt(name, err) => format!("name: {:?}, err: {:?}", name, err),
            }
        )
    }
}

impl Error for ConfigError {}

impl From<serde_json::error::Error> for ConfigError {
    fn from(err: serde_json::error::Error) -> Self {
        ConfigError::JsonParse(err)
    }
}

#[derive(Debug)]
pub enum JsonType {
    //Number,
    String,
    Array,
}

impl Display for JsonType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            //JsonType::Number => write!(f, "Number"),
            JsonType::String => write!(f, "String"),
            JsonType::Array => write!(f, "Array"),
        }
    }
}

#[derive(Debug)]
pub struct ParseColorError {
    message: String,
}

impl ParseColorError {
    pub fn new(str_value: &str, err: ParseIntError) -> Self {
        Self { message: format!("Invalid color value: {} ({})", str_value, err) }
    }
}

impl Error for ParseColorError {}

impl Display for ParseColorError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(self.message.as_str())
    }
}
