use crate::auto_color;
use crate::cli_app;
use crate::cli_app::Style;
use crate::imagery::RGB;
use crate::inout::ToJsonString;
use crate::pins;
use crate::style;
use image::GenericImageView;

// Create an image of the string art and output the knob positions and sequence
pub fn create_string() {
    let mut args = cli_app::parse_args();

    let image = args.image();
    let height = image.height();
    let width = image.width();

    let (foreground_colors, background_color) = match args.style {
        Style::Manual => (args.foreground_colors, args.background_color),
        Style::BlackOnWhite => (vec![RGB::BLACK], RGB::WHITE),
        Style::WhiteOnBlack => (vec![RGB::WHITE], RGB::BLACK),
        Style::AutoColor {
            auto_fg_count,
            ref manual_foregrounds,
            manual_background,
        } => auto_color::fg_and_bg(auto_fg_count, manual_foregrounds, manual_background, &image),
    };

    args.foreground_colors = foreground_colors;
    args.background_color = background_color;

    if args.verbosity > 1 {
        println!("Running with arguments: {:?}", args);
    }

    let pins = pins::generate(&args.pin_arrangement, args.pin_count, width, height);

    let data = style::color_on_custom_a(pins, args, image);

    if let Some(data_filepath) = &data.args.data_filepath {
        std::fs::write(data_filepath, data.to_json_string()).expect("Unable to write file");
    }
}
