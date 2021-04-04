mod cli_app;
mod generate_pins;
mod geometry;
mod imagery;
mod inout;
mod optimum;
mod style;

use geometry::Point;
use image::GenericImageView;
use imagery::RefImage;
use inout::Data;
use inout::ToJsonString;
use std::time::Instant;
use style::white_on_black;

// Create an image of the string art and output the knob positions and sequence
pub fn create_string() {
    let args = cli_app::parse_args();
    let image = args.image();
    let height = image.height();
    let width = image.width();

    let pins = match &args.pin_arrangement[..] {
        "perimeter" => generate_pins::perimeter(args.pin_count, width, height),
        "grid" => generate_pins::grid(args.pin_count, width, height),
        "circle" => generate_pins::circle(args.pin_count, width, height),
        "random" => generate_pins::random(args.pin_count, width, height),
        a => panic!("That's not a valid pin arrangement: {}", a),
    };

    let data_filepath_option = args.data_filepath.clone();

    let data = white_on_black::run(pins, args, image, Instant::now());

    if let Some(data_filepath) = data_filepath_option {
        std::fs::write(data_filepath, data.to_json_string()).expect("Unable to write file");
    }
}
