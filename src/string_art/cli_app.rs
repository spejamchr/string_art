use clap::{App, Arg, ArgMatches};
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
        .expect("This should have passed validation already")
}

pub fn parse_args() -> Args {
    let matches = get_matches();

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

fn get_matches() -> ArgMatches<'static> {
    App::new("strings")
        .version("0.1.0")
        .about("Transform an image into string art")
        .arg(
            Arg::with_name("image_filepath")
                .value_name("IMAGE_FILEPATH")
                .takes_value(true)
                .required(true)
                .help("Path to the image that will be rendered with strings"),
        )
        .arg(
            Arg::with_name("output_filepath")
                .value_name("FILEPATH")
                .short("o")
                .long("output-filepath")
                .takes_value(true)
                .help("Location to save generated string image"),
        )
        .arg(
            Arg::with_name("pins_filepath")
                .value_name("FILEPATH")
                .short("p")
                .long("pins-filepath")
                .takes_value(true)
                .help("Location to save image of pin locations"),
        )
        .arg(
            Arg::with_name("data_filepath")
                .value_name("FILEPATH")
                .short("d")
                .long("data-filepath")
                .takes_value(true)
                .help("Location for ouput of JSON with operation information that includes: argument values, starting and ending image scores, pin locations, and a list of line segments between pins that form the final image")
        )
        .arg(
            Arg::with_name("max_strings")
                .value_name("INTEGER")
                .short("m")
                .long("max-strings")
                .takes_value(true)
                .default_value("4294967295") // u32::MAX
                .validator(|f| {
                    f.parse::<usize>()
                        .map(|_| ())
                        .map_err(|e| format!("{:?}", e))
                })
                .help("The maximum number of strings in the finished work"),
        )
        .arg(
            Arg::with_name("step_size")
                .value_name("FLOAT")
                .short("s")
                .long("step-size")
                .takes_value(true)
                .default_value("1")
                .validator(|f| {
                        f.parse::<f64>()
                            .map(|_| ())
                            .map_err(|e| format!("{:?}", e))
                })
                .help("Used when calculating a string's antialiasing. Smaller values -> finer antialiasing")
        )
        .arg(
            Arg::with_name("string_alpha")
                .value_name("FLOAT")
                .short("a")
                .long("string-alpha")
                .takes_value(true)
                .default_value("1")
                .validator(|f| {
                        f.parse::<f64>()
                            .map_err(|e| format!("{:?}", e))
                            .and_then(|i|
                                if i > 0.0 && i <= 1.0 {
                                    Ok(i)
                                } else {
                                    Err(format!("{} is outside the range (0, 1]", i))
                                }
                            )
                            .map(|_| ())
                })
                .help("How opaque each string is: 1 is entirely opaque.")
        )
        .arg(
            Arg::with_name("pin_count")
                .value_name("INTEGER")
                .short("c")
                .long("pin-count")
                .takes_value(true)
                .default_value("200")
                .validator(|f| {
                        f.parse::<u32>()
                            .map(|_| ())
                            .map_err(|e| format!("{:?}", e))
                })
                .help("How many pins should be used in creating the image (approximately)")
        )
        .arg(
            Arg::with_name("pin_arrangement")
                .value_name("ARRANGEMENT")
                .short("r")
                .long("pin-arrangement")
                .takes_value(true)
                .possible_values(&["perimeter", "grid", "circle", "random"])
                .default_value("perimeter")
                .help("Should the pins be arranged on the image's perimeter, or in a grid across the entire image, or in the largest possible centered circle, or scattered randomly?")
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Output debugging messages")
        )
        .get_matches()
}
