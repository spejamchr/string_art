use crate::imagery::RGB;
use clap::{load_yaml, App, ArgMatches};
use image::io::Reader as ImageReader;
use std::collections::HashSet;

/// The validated arguments passed in by the user
#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub input_filepath: String,
    pub output_filepath: Option<String>,
    pub pins_filepath: Option<String>,
    pub data_filepath: Option<String>,
    pub gif_filepath: Option<String>,
    pub max_strings: usize,
    pub step_size: f64,
    pub string_alpha: f64,
    pub pin_count: u32,
    pub pin_arrangement: String,
    pub style: Style,
    pub foreground_colors: Vec<RGB>,
    pub background_color: RGB,
    pub verbosity: u64,
}

fn string_arg(matches: &ArgMatches, name: &str) -> String {
    matches
        .value_of(name)
        .unwrap_or_else(|| panic!("Required or default value for {}", name))
        .to_owned()
}

fn opt_string_arg(matches: &ArgMatches, name: &str) -> Option<String> {
    matches.value_of(name).map(|s| s.to_owned())
}

fn number_arg<E: std::fmt::Debug, T: std::str::FromStr<Err = E>>(
    matches: &ArgMatches,
    name: &str,
) -> T {
    matches
        .value_of(name)
        .unwrap_or_else(|| panic!("There is a default for {}", name))
        .parse::<T>()
        .unwrap_or_else(|_| panic!("Argument '{}' was not a valid number", name))
}

// Parses a color hex code of the form '#RRGGBB..' into an instance of 'RGB'
fn parse_rgb(hex_code: &str) -> RGB {
    let error = |_| panic!(format!("Invalid hex code: '{}'", hex_code));
    let r: u8 = u8::from_str_radix(&hex_code[1..3], 16).unwrap_or_else(error);
    let g: u8 = u8::from_str_radix(&hex_code[3..5], 16).unwrap_or_else(error);
    let b: u8 = u8::from_str_radix(&hex_code[5..7], 16).unwrap_or_else(error);
    RGB::new(r, g, b)
}

fn parse_rgbs(matches: &ArgMatches, name: &str) -> Vec<RGB> {
    matches
        .values_of(name)
        .map(|v| v.map(parse_rgb).collect())
        .unwrap_or_else(Vec::new)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Manual,
    WhiteOnBlack,
    BlackOnWhite,
    AutoColor {
        auto_fg_count: usize,
        manual_foregrounds: HashSet<RGB>,
        manual_background: Option<RGB>,
    },
}

fn from_bool<T>(b: bool) -> impl FnOnce(T) -> Option<T> {
    move |v: T| if b { Some(v) } else { None }
}

fn parse_manual_background(matches: &ArgMatches) -> Option<RGB> {
    matches
        .value_of("background_color")
        .and_then(from_bool(matches.occurrences_of("background_color") > 0))
        .map(parse_rgb)
}

fn parse_manual_foregrounds(matches: &ArgMatches) -> HashSet<RGB> {
    matches
        .values_of("foreground_color")
        .and_then(from_bool(matches.occurrences_of("foreground_color") > 0))
        .map(|v| v.map(parse_rgb).collect())
        .unwrap_or_else(HashSet::new)
}

fn parse_style(matches: &clap::ArgMatches) -> Style {
    match (
        matches.is_present("white_on_black"),
        matches.is_present("black_on_white"),
        matches.is_present("auto_color"),
    ) {
        (true, _, _) => Style::WhiteOnBlack,
        (_, true, _) => Style::BlackOnWhite,
        (_, _, true) => Style::AutoColor {
            auto_fg_count: number_arg(&matches, "auto_color"),
            manual_foregrounds: parse_manual_foregrounds(&matches),
            manual_background: parse_manual_background(&matches),
        },
        _ => Style::Manual,
    }
}

pub fn parse_args() -> Args {
    let yaml = load_yaml!("cli_args.yml");
    App::from_yaml(yaml).get_matches().into()
}

impl Args {
    pub fn image(&self) -> image::DynamicImage {
        ImageReader::open(&self.input_filepath)
            .unwrap_or_else(|_| panic!("Could not open input file '{}'", self.input_filepath))
            .decode()
            .expect("Corrupted file")
    }
}

impl From<clap::ArgMatches<'_>> for Args {
    fn from(matches: clap::ArgMatches) -> Self {
        Self {
            input_filepath: string_arg(&matches, "input_filepath"),
            output_filepath: opt_string_arg(&matches, "output_filepath"),
            pins_filepath: opt_string_arg(&matches, "pins_filepath"),
            data_filepath: opt_string_arg(&matches, "data_filepath"),
            gif_filepath: opt_string_arg(&matches, "gif_filepath"),
            max_strings: number_arg(&matches, "max_strings"),
            step_size: number_arg(&matches, "step_size"),
            string_alpha: number_arg(&matches, "string_alpha"),
            pin_count: number_arg(&matches, "pin_count"),
            pin_arrangement: string_arg(&matches, "pin_arrangement"),
            style: parse_style(&matches),
            foreground_colors: parse_rgbs(&matches, "foreground_color"),
            background_color: parse_rgb(&string_arg(&matches, "background_color")),
            verbosity: matches.occurrences_of("verbose"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn input_filepath() -> String {
        "test.png".to_owned()
    }

    fn default_args() -> Args {
        Args {
            input_filepath: input_filepath(),
            output_filepath: None,
            pins_filepath: None,
            data_filepath: None,
            gif_filepath: None,
            max_strings: u32::MAX as usize,
            step_size: 1.0,
            string_alpha: 1.0,
            pin_count: 200,
            pin_arrangement: "perimeter".to_owned(),
            style: Style::Manual,
            foreground_colors: vec![RGB::WHITE],
            background_color: RGB::BLACK,
            verbosity: 0,
        }
    }

    #[test]
    fn test_errors_without_input_filepath() {
        let yaml = load_yaml!("cli_args.yml");
        let matches: Result<_, _> = App::from_yaml(yaml).get_matches_from_safe(vec!["string_art"]);
        assert!(matches.is_err());
    }

    #[test]
    fn test_no_error_with_input_filepath() {
        let yaml = load_yaml!("cli_args.yml");
        let matches: Result<_, _> = App::from_yaml(yaml).get_matches_from_safe(vec![
            "string_art",
            "--input-filepath",
            "test.png",
        ]);
        assert!(matches.is_ok());
        let _args: Args = App::from_yaml(yaml)
            .get_matches_from(vec!["string_art", "--input-filepath", "test.png"])
            .into();
    }

    #[test]
    fn test_defaults() {
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec!["string_art", "--input-filepath", &input_filepath()])
            .into();
        assert_eq!(default_args(), args);
    }

    #[test]
    fn test_output_filepath() {
        let output_filepath = "output.png".to_owned();
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--output-filepath",
                &output_filepath,
            ])
            .into();
        assert_eq!(Some(output_filepath), args.output_filepath);
    }

    #[test]
    fn test_pins_filepath() {
        let pins_filepath = "pins.png".to_owned();
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--pins-filepath",
                &pins_filepath,
            ])
            .into();
        assert_eq!(Some(pins_filepath), args.pins_filepath);
    }

    #[test]
    fn test_data_filepath() {
        let data_filepath = "data.json".to_owned();
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--data-filepath",
                &data_filepath,
            ])
            .into();
        assert_eq!(Some(data_filepath), args.data_filepath);
    }

    #[test]
    fn test_gif_filepath() {
        let gif_filepath = "test.gif".to_owned();
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--gif-filepath",
                &gif_filepath,
            ])
            .into();
        assert_eq!(Some(gif_filepath), args.gif_filepath);
    }

    #[test]
    fn test_max_strings() {
        let max_strings = 10;
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--max-strings",
                &max_strings.to_string(),
            ])
            .into();
        assert_eq!(max_strings, args.max_strings);
    }

    #[test]
    fn test_step_size() {
        let step_size = 0.83;
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--step-size",
                &step_size.to_string(),
            ])
            .into();
        assert_eq!(step_size, args.step_size);
    }

    #[test]
    fn test_string_alpha() {
        let string_alpha = 0.83;
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--string-alpha",
                &string_alpha.to_string(),
            ])
            .into();
        assert_eq!(string_alpha, args.string_alpha);
    }

    #[test]
    fn test_pin_count() {
        let pin_count = 12;
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--pin-count",
                &pin_count.to_string(),
            ])
            .into();
        assert_eq!(pin_count, args.pin_count);
    }

    #[test]
    fn test_pin_arrangement() {
        let pin_arrangement = "random".to_owned();
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--pin-arrangement",
                &pin_arrangement,
            ])
            .into();
        assert_eq!(pin_arrangement, args.pin_arrangement);
    }

    #[test]
    fn test_background_color() {
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--background-color",
                "#0000FF",
            ])
            .into();
        assert_eq!(RGB::new(0, 0, 255), args.background_color);
    }

    #[test]
    fn test_foreground_color() {
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--foreground-color",
                "#0000FF",
                "--foreground-color",
                "#00FF00",
            ])
            .into();
        assert_eq!(
            vec![RGB::new(0, 0, 255), RGB::new(0, 255, 0)],
            args.foreground_colors
        );
    }

    #[test]
    fn test_style() {
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--white-on-black",
            ])
            .into();
        assert_eq!(Style::WhiteOnBlack, args.style);
    }

    #[test]
    fn test_auto_color() {
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--auto-color",
                "2",
            ])
            .into();
        assert_eq!(
            Style::AutoColor {
                auto_fg_count: 2,
                manual_background: None,
                manual_foregrounds: HashSet::new()
            },
            args.style
        );
    }

    #[test]
    fn test_auto_color_with_manual_fg_and_bg() {
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--auto-color",
                "2",
                "--background-color",
                "#FFFFFF",
                "--foreground-color",
                "#000000",
            ])
            .into();
        assert_eq!(
            Style::AutoColor {
                auto_fg_count: 2,
                manual_background: Some(RGB::WHITE),
                manual_foregrounds: vec![RGB::BLACK].into_iter().collect()
            },
            args.style
        );
    }

    #[test]
    fn test_verbosity() {
        let yaml = load_yaml!("cli_args.yml");
        let args: Args = App::from_yaml(yaml)
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--verbose",
                "--verbose",
            ])
            .into();
        assert_eq!(2, args.verbosity);
    }
}
