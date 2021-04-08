use crate::cli_app;
use crate::generate_pins;
use crate::inout::ToJsonString;
use crate::style::black_on_white;
use crate::style::explicit_color;
use crate::style::white_on_black;
use image::GenericImageView;

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

    let data = match &args.style[..] {
        "white-on-black" => white_on_black::run(pins, args, &image),
        "black-on-white" => black_on_white::run(pins, args, &image),
        "colors" => explicit_color::run(pins, args, &image),
        t => panic!("That's not a valid style: {}", t),
    };

    if let Some(data_filepath) = data_filepath_option {
        std::fs::write(data_filepath, data.to_json_string()).expect("Unable to write file");
    }
}
