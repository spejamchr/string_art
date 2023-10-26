use crate::{
    auto_color::{fg_and_bg, AutoColor},
    imagery::Rgb,
    pins::PinArrangement,
};
use clap::{builder::ArgPredicate, error::ErrorKind, Parser};
use image::io::Reader as ImageReader;
use serde::Serialize;
use std::{collections::HashSet, str::FromStr};

const DEFAULT_BG: &str = "#000000";
const DEFAULT_FG: &str = "#FFFFFF";

/// The validated arguments passed in by the user
#[derive(Debug, Clone, PartialEq, Serialize, Parser)]
#[command(version, about, long_about = None, max_term_width(100))]
pub struct Cli {
    /// Path to the image that will be rendered with strings.
    #[arg(short = 'i', long)]
    pub input_filepath: String,

    /// Location to save generated string image.
    #[arg(short = 'o', long)]
    pub output_filepath: Option<String>,

    /// Location to save image of pin locations.
    #[arg(short = 'p', long)]
    pub pins_filepath: Option<String>,

    /// The script will write operation information as a JSON file if this filepath is given. The
    /// operation information includes argument values, starting and ending image scores, pin
    /// locations, and a list of line segments between pins that form the final image.
    #[arg(short = 'd', long)]
    pub data_filepath: Option<String>,

    /// Location to save a gif of the creation process.
    #[arg(short = 'g', long)]
    pub gif_filepath: Option<String>,

    /// The maximum number of strings in the finished work.
    #[arg(short = 'm', long, default_value(usize::MAX.to_string()), hide_default_value(true))]
    pub max_strings: usize,

    /// Used when calculating a string's antialiasing. Smaller values -> finer antialiasing.
    #[arg(short = 's', long, default_value("1.0"))]
    pub step_size: f64,

    /// How opaque or thin each string is. `1` is entirely opaque, `0` is invisible.
    #[arg(short = 'a', long, default_value("0.2"))]
    pub string_alpha: f64,

    /// How many pins should be used in creating the image (approximately).
    #[arg(short = 'c', long, default_value("200"))]
    pub pin_count: u32,

    /// Should the pins be arranged on the image's perimeter, or in a grid across the entire image,
    /// or in the largest possible centered circle, or scattered randomly?
    #[arg(short = 'r', long, default_value("perimeter"))]
    pub pin_arrangement: PinArrangement,

    /// An RGB color in hex format `#RRGGBB` specifying the color of the background.
    #[arg(
        short = 'b',
        long,
        default_value(DEFAULT_BG),
        default_value_if("auto_color", ArgPredicate::IsPresent, None)
    )]
    pub background_color: Option<Rgb>,

    /// An RGB color in hex format `#RRGGBB` specifying the color of a string to use. Can be
    /// specified multiple times to specify multiple colors of strings.
    #[arg(
        short = 'f',
        long,
        default_value(DEFAULT_FG),
        default_value_if("auto_color", ArgPredicate::IsPresent, None)
    )]
    pub foreground_color: Option<Vec<Rgb>>,

    /// Draw with this many automatically chosen foreground colors on an automatically chosen
    /// background color.
    ///
    /// If --background-color is provided, use that.
    ///
    /// If --foreground-color is provided, use that in addition to this many additional
    /// automatically chosen foreground colors.
    #[arg(short = 'u', long)]
    pub auto_color: Option<usize>,

    /// Output debugging messages. Pass multiple times for more verbose logging.
    #[arg(short = 'v', long, action(clap::ArgAction::Count))]
    pub verbose: u8,
}

pub fn parse_args() -> Args {
    Cli::parse().into()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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
    pub pin_arrangement: PinArrangement,
    pub auto_color: Option<AutoColor>,
    pub foreground_colors: HashSet<Rgb>,
    pub background_color: Rgb,
    pub verbosity: u8,
    #[serde(skip)]
    pub image: image::DynamicImage,
}

impl Cli {
    pub fn image(&self) -> image::DynamicImage {
        ImageReader::open(&self.input_filepath)
            .unwrap_or_else(|_| {
                clap::Command::new("input_filepath")
                    .error(
                        ErrorKind::Io,
                        format!(
                            "The input filepath '{}' could not be opened",
                            &self.input_filepath
                        ),
                    )
                    .exit()
            })
            .decode()
            .unwrap_or_else(|_| {
                clap::Command::new("input_filepath")
                    .error(
                        ErrorKind::Io,
                        format!(
                            "The input filepath '{}' could not be decoded",
                            &self.input_filepath
                        ),
                    )
                    .exit()
            })
    }
}

impl From<Cli> for Args {
    fn from(cli: Cli) -> Self {
        let image = cli.image();
        let auto_color = cli.auto_color.map(|_| AutoColor::from(&cli));
        let (foreground_colors, background_color) = match &auto_color {
            Some(ac) => fg_and_bg(ac, &image),
            None => (
                cli.foreground_color
                    .unwrap_or_else(|| vec![Rgb::from_str(DEFAULT_FG).unwrap()])
                    .into_iter()
                    .collect(),
                cli.background_color
                    .unwrap_or_else(|| Rgb::from_str(DEFAULT_BG).unwrap()),
            ),
        };

        Self {
            input_filepath: cli.input_filepath,
            output_filepath: cli.output_filepath,
            pins_filepath: cli.pins_filepath,
            data_filepath: cli.data_filepath,
            gif_filepath: cli.gif_filepath,
            max_strings: cli.max_strings,
            step_size: cli.step_size,
            string_alpha: cli.string_alpha,
            pin_count: cli.pin_count,
            pin_arrangement: cli.pin_arrangement,
            auto_color,
            foreground_colors,
            background_color,
            verbosity: cli.verbose,
            image,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn input_filepath() -> String {
        "test.png".to_owned()
    }

    #[test]
    fn test_errors_without_input_filepath() {
        let matches: Result<_, _> = Cli::try_parse_from(vec!["string_art"]);
        assert!(matches.is_err());
    }

    #[test]
    fn test_no_error_with_input_filepath() {
        let matches: Result<_, _> =
            Cli::try_parse_from(vec!["string_art", "--input-filepath", &input_filepath()]);
        assert!(matches.is_ok());
    }

    #[test]
    fn test_output_filepath() {
        let output_filepath = "output.png".to_owned();
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--output-filepath",
            &output_filepath,
        ]);
        assert_eq!(Some(output_filepath), cli.output_filepath);
    }

    #[test]
    fn test_pins_filepath() {
        let pins_filepath = "pins.png".to_owned();
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--pins-filepath",
            &pins_filepath,
        ]);
        assert_eq!(Some(pins_filepath), cli.pins_filepath);
    }

    #[test]
    fn test_data_filepath() {
        let data_filepath = "data.json".to_owned();
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--data-filepath",
            &data_filepath,
        ]);
        assert_eq!(Some(data_filepath), cli.data_filepath);
    }

    #[test]
    fn test_gif_filepath() {
        let gif_filepath = "test.gif".to_owned();
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--gif-filepath",
            &gif_filepath,
        ]);
        assert_eq!(Some(gif_filepath), cli.gif_filepath);
    }

    #[test]
    fn test_max_strings() {
        let max_strings = 10;
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--max-strings",
            &max_strings.to_string(),
        ]);
        assert_eq!(max_strings, cli.max_strings);
    }

    #[test]
    fn test_step_size() {
        let step_size = 0.83;
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--step-size",
            &step_size.to_string(),
        ]);
        assert_eq!(step_size, cli.step_size);
    }

    #[test]
    fn test_string_alpha() {
        let string_alpha = 0.83;
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--string-alpha",
            &string_alpha.to_string(),
        ]);
        assert_eq!(string_alpha, cli.string_alpha);
    }

    #[test]
    fn test_pin_count() {
        let pin_count = 12;
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--pin-count",
            &pin_count.to_string(),
        ]);
        assert_eq!(pin_count, cli.pin_count);
    }

    #[test]
    fn test_pin_arrangement() {
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--pin-arrangement",
            "random",
        ]);
        assert_eq!(PinArrangement::Random, cli.pin_arrangement);
    }

    #[test]
    fn test_background_color() {
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--background-color",
            "#0000FF",
        ]);
        assert_eq!(Some(Rgb::new(0, 0, 255)), cli.background_color);
    }

    #[test]
    fn test_foreground_color() {
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--foreground-color",
            "#0000FF",
            "--foreground-color",
            "#00FF00",
        ]);
        assert_eq!(
            Some(vec![Rgb::new(0, 0, 255), Rgb::new(0, 255, 0)]),
            cli.foreground_color
        );
    }

    #[test]
    fn test_auto_color() {
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--auto-color",
            "2",
        ]);
        assert_eq!(
            AutoColor {
                auto_fg_count: 2,
                manual_background: None,
                manual_foregrounds: HashSet::new()
            },
            AutoColor::from(&cli)
        );
    }

    #[test]
    fn test_two_foreground_colors() {
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--foreground-color",
            "#FFFFFF",
            "--foreground-color",
            "#FF0000",
        ]);
        assert_eq!(
            Some(vec![Rgb::WHITE, Rgb { r: 255, g: 0, b: 0 }]),
            cli.foreground_color
        )
    }

    #[test]
    fn test_auto_color_with_manual_fg_and_bg() {
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--auto-color",
            "2",
            "--background-color",
            "#FFFFFF",
            "--foreground-color",
            "#000000",
        ]);
        assert_eq!(
            AutoColor {
                auto_fg_count: 2,
                manual_background: Some(Rgb::WHITE),
                manual_foregrounds: vec![Rgb::BLACK].into_iter().collect()
            },
            AutoColor::from(&cli)
        );
    }

    #[test]
    fn test_verbosity() {
        let cli = Cli::parse_from(vec![
            "string_art",
            "--input-filepath",
            &input_filepath(),
            "--verbose",
            "--verbose",
        ]);
        assert_eq!(2, cli.verbose);
    }
}
