use crate::imagery::RGB;
use clap::{load_yaml, App, ArgMatches};
use image::io::Reader as ImageReader;
use std::collections::HashSet;

/// The validated arguments passed in by the user
#[derive(Debug, Clone)]
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
        .to_string()
}

fn opt_string_arg(matches: &ArgMatches, name: &str) -> Option<String> {
    matches.value_of(name).map(|s| s.to_string())
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

#[derive(Debug, Clone)]
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

pub fn parse_args() -> Args {
    let yaml = load_yaml!("cli_args.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let style = match (
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
    };

    Args {
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
        style,
        foreground_colors: parse_rgbs(&matches, "foreground_color"),
        background_color: parse_rgb(&string_arg(&matches, "background_color")),
        verbosity: matches.occurrences_of("verbose"),
    }
}

impl Args {
    pub fn image(&self) -> image::DynamicImage {
        ImageReader::open(&self.input_filepath)
            .unwrap_or_else(|_| panic!("Could not open input file '{}'", self.input_filepath))
            .decode()
            .expect("Corrupted file")
    }
}
