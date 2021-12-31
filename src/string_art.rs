use crate::auto_color;
use crate::cli_app;
use crate::image::GenericImageView;
use crate::pins;
use crate::style;

// Create an image of the string art and output the knob positions and sequence
pub fn create_string() {
    let mut args = cli_app::parse_args();

    let image = args.image();
    let height = image.height();
    let width = image.width();

    let (foreground_colors, background_color) = match &args.auto_color {
        Some(auto_color) => auto_color::fg_and_bg(auto_color, &image),
        None => (args.foreground_colors, args.background_color),
    };

    args.foreground_colors = foreground_colors;
    args.background_color = background_color;

    if args.verbosity > 1 {
        println!("Running with arguments: {:?}", args);
    }

    let pins = pins::generate(&args.pin_arrangement, args.pin_count, width, height);

    let data = style::color_on_custom(pins, args, image);

    if let Some(data_filepath) = &data.args.data_filepath {
        std::fs::write(data_filepath, serde_json::to_string(&data).unwrap())
            .expect("Unable to write file");
    }
}
