use ansi_term::Color;
use regex::Regex;

pub trait Processor {
    fn process_line(&mut self, line: &str) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct PatternColor {
    pub regex: Regex,
    pub color: Color,
}

#[derive(Debug, Clone, Default)]
pub struct PatternColors(pub Vec<PatternColor>);

impl PatternColors {
    pub fn find_color_for_pattern<T: AsRef<str>>(&self, line: T) -> Option<Color> {
        self.0
            .iter()
            .find(|pattern| pattern.regex.is_match(line.as_ref()))
            .map(|pattern| pattern.color)
    }
}

#[derive(Debug, Clone)]
pub struct Colorize {
    pattern_colors: PatternColors,
    current_color: Option<Color>,
}

impl Colorize {
    pub fn new(pattern_colors: PatternColors) -> Self {
        Self {
            pattern_colors,
            current_color: None,
        }
    }
}

impl Processor for Colorize {
    fn process_line(&mut self, line: &str) -> Option<String> {
        if let Some(color) = self.pattern_colors.find_color_for_pattern(line) {
            self.current_color = Some(color);
        }

        Some(self.current_color.unwrap_or(Color::White).paint(line).to_string())
    }
}

#[derive(Debug, Clone)]
pub struct EventPatterns {
    pub start_regex: Regex,
    pub end_regex: Regex,
    pub color: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct EventProcessor {
    event_patterns: EventPatterns,
    current_event: Option<String>,
}

impl EventProcessor {
    pub fn new(event_patterns: EventPatterns) -> Self {
        Self {
            event_patterns,
            current_event: None,
        }
    }
}

impl Processor for EventProcessor {
    fn process_line(&mut self, line: &str) -> Option<String> {
        match &mut self.current_event {
            Some(_) if self.event_patterns.end_regex.is_match(line) => {
                let event = format!("Event:\n{}{}\n", self.current_event.take().unwrap(), line);
                Some(
                    self.event_patterns
                        .color
                        .map(|color| color.paint(&event).to_string())
                        .unwrap_or(event),
                )
            }
            Some(event) => {
                *event += format!("{}\n", line).as_str();
                None
            }
            None => {
                if self.event_patterns.start_regex.is_match(line) {
                    self.current_event = Some(format!("{}\n", line));
                }

                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateProcessor {
    pub(crate) regex: Regex,
    pub(crate) color: Option<Color>,
}

impl StateProcessor {
    pub fn new(regex: Regex, color: Option<Color>) -> Self {
        Self { regex, color }
    }
}

impl Processor for StateProcessor {
    fn process_line(&mut self, line: &str) -> Option<String> {
        if self.regex.is_match(line) {
            let state = format!("State: {}\n", line);
            Some(self.color.map(|color| color.paint(&state).to_string()).unwrap_or(state))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Colorize, EventPatterns, EventProcessor, PatternColor, PatternColors, Processor, StateProcessor};
    use ansi_term::Color;
    use regex::Regex;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        path::PathBuf,
    };

    const DATE_REGEX_STR: &'static str = r"[\d]{4}-[\d]{2}-[\d]{2} [\d]{2}:[\d]{2}:[\d]{2}";

    fn create_level_colors() -> PatternColors {
        PatternColors(vec![
            PatternColor {
                regex: Regex::new(format!("{} INFO ", DATE_REGEX_STR).as_str()).unwrap(),
                color: Color::Fixed(28),
            },
            PatternColor {
                regex: Regex::new(format!("{} WARN ", DATE_REGEX_STR).as_str()).unwrap(),
                color: Color::Fixed(24),
            },
            PatternColor {
                regex: Regex::new(format!("{} ERROR ", DATE_REGEX_STR).as_str()).unwrap(),
                color: Color::Fixed(88),
            },
        ])
    }

    #[test]
    fn test_colorize_lines() {
        let test_log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test.log");
        let file = File::open(test_log_path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut colorize = Colorize::new(create_level_colors());
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:00 INFO Start of log file")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:01 INFO Mouse left down at 0, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:02 INFO Mouse moved to 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:03 INFO Mouse left up at 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(24)
                    .paint("2020-01-01 10:00:03 WARN Invalid mouse coordinates 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:03 INFO Set state to options")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:04 INFO Mouse left down at 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:04 INFO Mouse moved to 10, 10")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:05 INFO Mouse left up at 10, 10")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:05 INFO Set state to main_menu")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(88)
                    .paint("2020-01-01 10:00:50 ERROR Failed to start application")
                    .to_string()
            )
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(Color::Fixed(88).paint("An unknown error occurred").to_string())
        );
        assert_eq!(
            colorize.process_line(lines.next().unwrap().unwrap().as_str()),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:01:00 INFO End of log file")
                    .to_string()
            )
        );
    }

    #[test]
    fn test_events() {
        let test_log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test.log");
        let file = File::open(test_log_path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut events = EventProcessor::new(EventPatterns {
            start_regex: Regex::new(format!(r"{} INFO Mouse left down at [\d]+, [\d]+", DATE_REGEX_STR).as_str())
                .unwrap(),
            end_regex: Regex::new(format!(r"{} INFO Mouse left up at [\d]+, [\d]+", DATE_REGEX_STR).as_str()).unwrap(),
            color: Some(Color::Fixed(28)),
        });

        for line in &mut lines {
            if let Some(event) = events.process_line(line.unwrap().as_str()) {
                assert_eq!(
                    event,
                    Color::Fixed(28)
                        .paint(
                            "Event:
2020-01-01 10:00:01 INFO Mouse left down at 0, 0
2020-01-01 10:00:02 INFO Mouse moved to 10, 0
2020-01-01 10:00:03 INFO Mouse left up at 10, 0
"
                        )
                        .to_string()
                );
                break;
            }
        }

        for line in &mut lines {
            if let Some(event) = events.process_line(line.unwrap().as_str()) {
                assert_eq!(
                    event,
                    Color::Fixed(28)
                        .paint(
                            "Event:
2020-01-01 10:00:04 INFO Mouse left down at 10, 0
2020-01-01 10:00:04 INFO Mouse moved to 10, 10
2020-01-01 10:00:05 INFO Mouse left up at 10, 10
"
                        )
                        .to_string()
                );
            }
        }
    }

    #[test]
    fn test_states() {
        let test_log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test.log");
        let file = File::open(test_log_path).unwrap();
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut states = StateProcessor::new(
            Regex::new(format!("{} INFO Set state to (.*)", DATE_REGEX_STR).as_str()).unwrap(),
            Some(Color::Fixed(28)),
        );

        for line in &mut lines {
            if let Some(state) = states.process_line(line.unwrap().as_str()) {
                assert_eq!(
                    state,
                    Color::Fixed(28)
                        .paint("State: 2020-01-01 10:00:03 INFO Set state to options\n")
                        .to_string()
                );
                break;
            }
        }

        for line in &mut lines {
            if let Some(state) = states.process_line(line.unwrap().as_str()) {
                assert_eq!(
                    state,
                    Color::Fixed(28)
                        .paint("State: 2020-01-01 10:00:05 INFO Set state to main_menu\n")
                        .to_string()
                );
                break;
            }
        }
    }
}
