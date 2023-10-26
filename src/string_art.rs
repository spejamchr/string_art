use crate::cli_app;
use crate::geometry::Point;
use crate::pins;
use crate::style;

// Create an image of the string art and output the knob positions and sequence
pub fn create_string() {
    let args = cli_app::parse_args();

    let height = args.image.height();
    let width = args.image.width();

    if args.verbosity > 1 {
        println!(
            "Running with arguments: {}",
            serde_json::to_string(&args).unwrap()
        );
    }

    let pins = pins::generate(&args.pin_arrangement, args.pin_count, width, height);

    if let Some(ref pins_filepath) = args.pins_filepath {
        draw_pin_crosshairs(width, height, &pins, pins_filepath);
    }

    let data = style::color_on_custom(pins, args);

    if let Some(data_filepath) = &data.args.data_filepath {
        std::fs::write(data_filepath, serde_json::to_string(&data).unwrap())
            .expect("Unable to write file");
    }
}

fn draw_pin_crosshairs(width: u32, height: u32, pins: &[Point], pins_filepath: &str) {
    let mut img = image::GrayImage::from_pixel(width, height, image::Luma([255]));
    for pin in pins {
        let side_length = 3;
        for x in pin.x.saturating_sub(side_length)..=pin.x.saturating_add(side_length) {
            if x > 0 && x < width {
                img.get_pixel_mut(x, pin.y)[0] = 0;
            }
        }
        for y in pin.y.saturating_sub(side_length)..=pin.y.saturating_add(side_length) {
            if y > 0 && y < height {
                img.get_pixel_mut(pin.x, y)[0] = 0;
            }
        }
    }
    img.save(pins_filepath)
        .unwrap_or_else(|_| panic!("Unable to create pin file at: '{}'", pins_filepath))
}
