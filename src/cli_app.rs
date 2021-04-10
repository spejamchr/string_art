use crate::imagery::RGB;
use clap::{load_yaml, App, ArgMatches};
use image::io::Reader as ImageReader;

/// The validated arguments passed in by the user
#[derive(Debug, Clone)]
pub struct Args {
    input_filepath: String,
    pub output_filepath: Option<String>,
    pub pins_filepath: Option<String>,
    pub data_filepath: Option<String>,
    pub gif_filepath: Option<String>,
    pub max_strings: usize,
    pub step_size: f64,
    pub string_alpha: f64,
    pub pin_count: u32,
    pub pin_arrangement: String,
    pub style: String,
    pub auto_color_limit: u32,
    pub rgbs: Vec<RGB>,
    pub verbosity: u64,
}

fn string_arg(matches: &ArgMatches, name: &str) -> String {
    matches
        .value_of(name)
        .expect("Required or default value")
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
        .expect("There is a default")
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

pub fn parse_args() -> Args {
    let yaml = load_yaml!("cli_args.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let args = Args {
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
        style: string_arg(&matches, "style"),
        auto_color_limit: number_arg(&matches, "auto_color_limit"),
        rgbs: parse_rgbs(&matches, "hex_color"),
        verbosity: matches.occurrences_of("verbose"),
    };

    if args.verbosity > 1 {
        println!("Running with arguments: {:?}", args);
    }

    args
}

impl Args {
    pub fn image(&self) -> image::DynamicImage {
        ImageReader::open(&self.input_filepath)
            .unwrap()
            .decode()
            .expect("Corrupted file")
    }
}
