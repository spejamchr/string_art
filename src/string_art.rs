use crate::cli_app;
use crate::generate_pins;
use crate::inout::ToJsonString;
use crate::style;
use image::GenericImageView;

// Create an image of the string art and output the knob positions and sequence
pub fn create_string() {
    let args = cli_app::parse_args();

    let pin_placer = match &args.pin_arrangement[..] {
        "perimeter" => generate_pins::perimeter,
        "grid" => generate_pins::grid,
        "circle" => generate_pins::circle,
        "random" => generate_pins::random,
        a => panic!("That's not a valid pin arrangement: {}", a),
    };

    let stringer = match &args.style[..] {
        "white-on-black" => style::white_on_black,
        "black-on-white" => style::black_on_white,
        "color-on-white" => style::color_on_white,
        "color-on-black" => style::color_on_black,
        "auto-color" => style::auto_color,
        t => panic!("That's not a valid style: {}", t),
    };

    let image = args.image();
    let height = image.height();
    let width = image.width();

    let pins = pin_placer(args.pin_count, width, height);

    let data = stringer(pins, args, image);

    if let Some(data_filepath) = &data.args.data_filepath {
        std::fs::write(data_filepath, data.to_json_string()).expect("Unable to write file");
    }
}
