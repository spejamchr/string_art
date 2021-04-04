use clap::{load_yaml, App, ArgMatches};
use image::io::Reader as ImageReader;

/// The validated arguments passed in by the user
#[derive(Debug, Clone)]
pub struct Args {
    image_filepath: String,
    pub output_filepath: Option<String>,
    pub pins_filepath: Option<String>,
    pub data_filepath: Option<String>,
    pub max_strings: usize,
    pub step_size: f64,
    pub string_alpha: f64,
    pub pin_count: u32,
    pub pin_arrangement: String,
    pub style: String,
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

pub fn parse_args() -> Args {
    let yaml = load_yaml!("cli_args.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let args = Args {
        image_filepath: string_arg(&matches, "image_filepath"),
        output_filepath: opt_string_arg(&matches, "output_filepath"),
        pins_filepath: opt_string_arg(&matches, "pins_filepath"),
        data_filepath: opt_string_arg(&matches, "data_filepath"),
        max_strings: number_arg(&matches, "max_strings"),
        step_size: number_arg(&matches, "step_size"),
        string_alpha: number_arg(&matches, "string_alpha"),
        pin_count: number_arg(&matches, "pin_count"),
        pin_arrangement: string_arg(&matches, "pin_arrangement"),
        style: string_arg(&matches, "style"),
        verbosity: matches.occurrences_of("verbose"),
    };

    if args.verbosity > 1 {
        println!("Running with arguments: {:?}", args);
    }

    args
}

impl Args {
    pub fn image(&self) -> image::DynamicImage {
        ImageReader::open(&self.image_filepath)
            .unwrap()
            .decode()
            .expect("Corrupted file")
    }
}
