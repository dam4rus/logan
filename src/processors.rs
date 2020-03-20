use ansi_term::Color;
use regex::Regex;
use std::io::{BufRead, Lines};

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

#[derive(Debug)]
pub struct ColorizeLines<B: BufRead> {
    pattern_colors: PatternColors,
    lines: Lines<B>,
    current_color: Option<Color>,
}

impl<B: BufRead> ColorizeLines<B> {
    pub fn new(pattern_colors: PatternColors, lines: Lines<B>) -> Self {
        Self {
            pattern_colors,
            lines,
            current_color: None,
        }
    }
}

impl<B: BufRead> Iterator for ColorizeLines<B> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().and_then(|line_result| line_result.ok()).map(|line| {
            if let Some(color) = self.pattern_colors.find_color_for_pattern(&line) {
                self.current_color = Some(color);
            }

            self.current_color.unwrap_or(Color::White).paint(line).to_string()
        })
    }
}

#[derive(Debug)]
pub struct Events<B: BufRead> {
    start_regex: Regex,
    end_regex: Regex,
    lines: Lines<B>,
    color: Option<Color>,
}

impl<B: BufRead> Events<B> {
    pub fn new(start_regex: Regex, end_regex: Regex, lines: Lines<B>, color: Option<Color>) -> Self {
        Self {
            start_regex,
            end_regex,
            lines,
            color,
        }
    }
}

impl<B: BufRead> Iterator for Events<B> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut event = None;
        while let Some(Ok(line)) = self.lines.next() {
            if self.start_regex.is_match(line.as_str()) {
                event = Some(line);
                break;
            }
        }

        let mut event = event.map(|line| line + "\n")?;
        while let Some(Ok(line)) = self.lines.next() {
            event += format!("{}\n", line).as_str();
            if self.end_regex.is_match(line.as_str()) {
                break;
            }
        }

        self.color.map(|color| color.paint(&event).to_string()).or(Some(event))
    }
}

#[derive(Debug)]
pub struct States<B: BufRead> {
    regex: Regex,
    capture_group: usize,
    lines: Lines<B>,
}

impl<B: BufRead> States<B> {
    pub fn new(regex: Regex, capture_group: usize, lines: Lines<B>) -> Self {
        Self {
            regex,
            capture_group,
            lines,
        }
    }
}

impl<B: BufRead> Iterator for States<B> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Ok(line)) = self.lines.next() {
            let captured_string = self
                .regex
                .captures(line.as_str())
                .and_then(|captures| captures.get(self.capture_group))
                .map(|group| group.as_str().to_owned());

            if let Some(state) = captured_string {
                return Some(state);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorizeLines, Events, PatternColors, PatternColor, States};
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
                regex: Regex::new(format!("{} DEBUG ", DATE_REGEX_STR).as_str()).unwrap(),
                color: Color::Fixed(28),
            },
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

        let mut colorize_lines = ColorizeLines::new(create_level_colors(), reader.lines());
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:00 INFO Start of log file")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:01 INFO Mouse left down at 0, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:02 INFO Mouse moved to 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:03 INFO Mouse left up at 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(24)
                    .paint("2020-01-01 10:00:03 WARN Invalid mouse coordinates 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:03 INFO Set state to options")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:04 INFO Mouse left down at 10, 0")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:04 INFO Mouse moved to 10, 10")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:05 INFO Mouse left up at 10, 10")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(28)
                    .paint("2020-01-01 10:00:05 INFO Set state to main_menu")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(
                Color::Fixed(88)
                    .paint("2020-01-01 10:00:50 ERROR Failed to start application")
                    .to_string()
            )
        );
        assert_eq!(
            colorize_lines.next(),
            Some(Color::Fixed(88).paint("An unknown error occurred").to_string())
        );
        assert_eq!(
            colorize_lines.next(),
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

        let mut events = Events::new(
            Regex::new(format!(r"{} INFO Mouse left down at [\d]+, [\d]+", DATE_REGEX_STR).as_str()).unwrap(),
            Regex::new(format!(r"{} INFO Mouse left up at [\d]+, [\d]+", DATE_REGEX_STR).as_str()).unwrap(),
            reader.lines(),
            Some(Color::Fixed(28)),
        );

        assert_eq!(
            events.next(),
            Some(
                Color::Fixed(28)
                    .paint(
                        "2020-01-01 10:00:01 INFO Mouse left down at 0, 0
2020-01-01 10:00:02 INFO Mouse moved to 10, 0
2020-01-01 10:00:03 INFO Mouse left up at 10, 0
"
                    )
                    .to_string()
            ),
        );
        assert_eq!(
            events.next(),
            Some(
                Color::Fixed(28)
                    .paint(
                        "2020-01-01 10:00:04 INFO Mouse left down at 10, 0
2020-01-01 10:00:04 INFO Mouse moved to 10, 10
2020-01-01 10:00:05 INFO Mouse left up at 10, 10
"
                    )
                    .to_string()
            ),
        );
    }

    #[test]
    fn test_states() {
        let test_log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test.log");
        let file = File::open(test_log_path).unwrap();
        let reader = BufReader::new(file);
        let mut states = States::new(
            Regex::new(format!("{} INFO Set state to (.*)", DATE_REGEX_STR).as_str()).unwrap(),
            1,
            reader.lines(),
        );

        assert_eq!(states.next(), Some(String::from("options")));
        assert_eq!(states.next(), Some(String::from("main_menu")));
    }
}
