use clap::{command, Command};

use crate::clap::Arg;
use crate::util;

fn valid_type<T: std::str::FromStr>(name: &str) -> impl Fn(String) -> Result<(), String> {
    let name = name.to_owned();
    move |f| {
        f.parse::<T>()
            .map(|_| ())
            .map_err(|_| format!("Expected {} but got \"{}\"", name, f))
    }
}

fn float_in_range(min: f64, max: f64) -> impl Fn(String) -> Result<(), String> {
    move |f| {
        f.parse::<f64>()
            .ok()
            .and_then(|f| util::from_bool(f > min && f <= max)(f))
            .map(|_| ())
            .ok_or_else(|| {
                format!(
                    "Expected Float on range {} < Float <= {} but got \"{}\"",
                    min, max, f
                )
            })
    }
}

pub fn create() -> Command {
    command!()
    .arg(Arg::new("input_filepath")
        .value_name("FILEPATH")
        .short('i')
        .long("input-filepath")
        .required(true)
        .help("Path to the image that will be rendered with strings.")
    )
    .arg(Arg::new("output_filepath")
        .value_name("FILEPATH")
        .short('o')
        .long("output-filepath")
        .help("Location to save generated string image.")
    )
    .arg(Arg::new("pins_filepath")
        .value_name("FILEPATH")
        .short('p')
        .long("pins-filepath")
        .help("Location to save image of pin locations.")
    )
    .arg(Arg::new("data_filepath")
        .value_name("FILEPATH")
        .short('d')
        .long("data-filepath")
        .help("The script will write operation information as a JSON file if this filepath is given. The operation information includes argument values, starting and ending image scores, pin locations, and a list of line segments between pins that form the final image.")
    )
    .arg(Arg::new("gif_filepath")
        .value_name("FILEPATH")
        .short('g')
        .long("gif-filepath")
        .help("Location to save a gif of the creation process")
    )
    .arg(Arg::new("max_strings")
        .value_name("INTEGER")
        .short('m')
        .long("max-strings")
        .help("The maximum number of strings in the finished work.")
    )
    .arg(Arg::new("step_size")
        .value_name("FLOAT")
        .short('s')
        .long("step-size")
        .default_value("1.0")
            .value_parser(0..=50)
        .help("Used when calculating a string's antialiasing. Smaller values -> finer antialiasing. [range: 0 < value <= 50]")
    )
    .arg(Arg::new("string_alpha")
        .value_name("FLOAT")
        .short('a')
        .long("string-alpha")
        .default_value("0.2")
            .value_parser(0..=1)
        .help("How opaque or thin each string is. `1` is entirely opaque, `0` is invisible. [range: 0 < value <= 1]")
    )
    .arg(Arg::new("pin_count")
        .value_name("INTEGER")
        .short('c')
        .long("pin-count")
        .default_value("200")
        .help("How many pins should be used in creating the image (approximately).")
    )
    .arg(Arg::new("pin_arrangement")
        .value_name("ARRANGEMENT")
        .short('r')
        .long("pin-arrangement")
            .value_parser(["perimeter", "grid", "circle", "random"])
        .default_value("perimeter")
        .help("Should the pins be arranged on the image's perimeter, or in a grid across the entire image, or in the largest possible centered circle, or scattered randomly?")
    )
    .arg(Arg::new("background_color")
        .value_name("HEX CODE")
        .short('b')
        .long("background-color")
        .default_value("#000000")
        .help("An RGB color in hex format `#RRGGBB` specifying the color of the background.")
    )
    .arg(Arg::new("foreground_color")
        .value_name("HEX CODE")
        .short('f')
        .long("foreground-color")
            .action(clap::ArgAction::Append)
        .default_value("#FFFFFF")
        .help("An RGB color in hex format `#RRGGBB` specifying the color of a string to use. Can be specified multiple times to specify multiple colors of strings.")
    )
    .arg(Arg::new("auto_color")
        .value_name("INTEGER")
        .short('u')
        .long("auto-color")
        .help("Draw with this many automatically chosen foreground colors on an automatically chosen background color.

If --background-color is provided, use that.

If --foreground-color is provided, use that in addition to this many additional automatically chosen foreground colors.")
    )
    .arg(Arg::new("verbose")
        .short('v')
        .long("verbose")
        .action(clap::ArgAction::Count)
        .help("Output debugging messages. Pass multiple times for more verbose logging.")
    )
}
