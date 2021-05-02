use crate::clap::value_t;
use crate::clap::values_t;
use crate::clap::ArgMatches;
use crate::image::io::Reader as ImageReader;
use crate::imagery::RGB;
use crate::pins::PinArrangement;
use crate::serde::Serialize;
use crate::util;
use std::collections::HashSet;

mod app;

/// The validated arguments passed in by the user
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
    pub foreground_colors: Vec<RGB>,
    pub background_color: RGB,
    pub verbosity: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutoColor {
    pub auto_fg_count: usize,
    pub manual_foregrounds: HashSet<RGB>,
    pub manual_background: Option<RGB>,
}

fn parse_manual_background(matches: &ArgMatches) -> Option<RGB> {
    matches
        .value_of("background_color")
        .and_then(util::from_bool(
            matches.occurrences_of("background_color") > 0,
        ))
        .map(|_| value_t!(matches, "background_color", RGB).unwrap())
}

fn parse_manual_foregrounds(matches: &ArgMatches) -> HashSet<RGB> {
    matches
        .values_of("foreground_color")
        .and_then(util::from_bool(
            matches.occurrences_of("foreground_color") > 0,
        ))
        .and_then(|_| values_t!(matches, "foreground_color", RGB).ok())
        .map(|v| v.into_iter().collect())
        .unwrap_or_else(HashSet::new)
}

fn parse_auto_color(matches: &ArgMatches) -> Option<AutoColor> {
    value_t!(matches, "auto_color", usize)
        .ok()
        .map(|count| AutoColor {
            auto_fg_count: count,
            manual_foregrounds: parse_manual_foregrounds(&matches),
            manual_background: parse_manual_background(&matches),
        })
}

pub fn parse_args() -> Args {
    app::create().get_matches().into()
}

impl Args {
    pub fn image(&self) -> image::DynamicImage {
        ImageReader::open(&self.input_filepath)
            .unwrap_or_else(|_| {
                clap::Error::value_validation_auto(format!(
                    "The input filepath '{}' could not be opened",
                    &self.input_filepath
                ))
                .exit()
            })
            .decode()
            .unwrap_or_else(|_| {
                clap::Error::value_validation_auto(format!(
                    "The input filepath '{}' could not be decoded",
                    &self.input_filepath
                ))
                .exit()
            })
    }
}

impl From<ArgMatches<'_>> for Args {
    fn from(matches: ArgMatches) -> Self {
        Self {
            input_filepath: value_t!(matches, "input_filepath", String).unwrap(),
            output_filepath: value_t!(matches, "output_filepath", String).ok(),
            pins_filepath: value_t!(matches, "pins_filepath", String).ok(),
            data_filepath: value_t!(matches, "data_filepath", String).ok(),
            gif_filepath: value_t!(matches, "gif_filepath", String).ok(),
            max_strings: value_t!(matches, "max_strings", usize).unwrap(),
            step_size: value_t!(matches, "step_size", f64).unwrap(),
            string_alpha: value_t!(matches, "string_alpha", f64).unwrap(),
            pin_count: value_t!(matches, "pin_count", u32).unwrap(),
            pin_arrangement: value_t!(matches, "pin_arrangement", PinArrangement).unwrap(),
            auto_color: parse_auto_color(&matches),
            foreground_colors: values_t!(matches.values_of("foreground_color"), RGB).unwrap(),
            background_color: value_t!(matches, "background_color", RGB).unwrap(),
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
            string_alpha: 0.2,
            pin_count: 200,
            pin_arrangement: PinArrangement::Perimeter,
            auto_color: None,
            foreground_colors: vec![RGB::WHITE],
            background_color: RGB::BLACK,
            verbosity: 0,
        }
    }

    #[test]
    fn test_errors_without_input_filepath() {
        let matches: Result<_, _> = app::create().get_matches_from_safe(vec!["string_art"]);
        assert!(matches.is_err());
    }

    #[test]
    fn test_no_error_with_input_filepath() {
        let matches: Result<_, _> = app::create().get_matches_from_safe(vec![
            "string_art",
            "--input-filepath",
            "test.png",
        ]);
        assert!(matches.is_ok());
        let _args: Result<Args, _> = matches.map(|a| a.into());
    }

    #[test]
    fn test_defaults() {
        let args: Args = app::create()
            .get_matches_from(vec!["string_art", "--input-filepath", &input_filepath()])
            .into();
        assert_eq!(default_args(), args);
    }

    #[test]
    fn test_output_filepath() {
        let output_filepath = "output.png".to_owned();
        let args: Args = app::create()
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
        let args: Args = app::create()
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
        let args: Args = app::create()
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
        let args: Args = app::create()
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
        let args: Args = app::create()
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
        let args: Args = app::create()
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
        let args: Args = app::create()
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
        let args: Args = app::create()
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
        let args: Args = app::create()
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--pin-arrangement",
                "random",
            ])
            .into();
        assert_eq!(PinArrangement::Random, args.pin_arrangement);
    }

    #[test]
    fn test_background_color() {
        let args: Args = app::create()
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
        let args: Args = app::create()
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
    fn test_auto_color() {
        let args: Args = app::create()
            .get_matches_from(vec![
                "string_art",
                "--input-filepath",
                &input_filepath(),
                "--auto-color",
                "2",
            ])
            .into();
        assert_eq!(
            Some(AutoColor {
                auto_fg_count: 2,
                manual_background: None,
                manual_foregrounds: HashSet::new()
            }),
            args.auto_color
        );
    }

    #[test]
    fn test_auto_color_with_manual_fg_and_bg() {
        let args: Args = app::create()
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
            Some(AutoColor {
                auto_fg_count: 2,
                manual_background: Some(RGB::WHITE),
                manual_foregrounds: vec![RGB::BLACK].into_iter().collect()
            }),
            args.auto_color
        );
    }

    #[test]
    fn test_verbosity() {
        let args: Args = app::create()
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
