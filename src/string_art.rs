mod cli_app;
mod generate_pins;
mod geometry;
mod imagery;
mod inout;
mod optimum;

use geometry::Line;
use geometry::Point;
use image::GenericImageView;
use imagery::RefImage;
use inout::Data;
use inout::ToJsonString;

// Create an image of the string art and output the knob positions and sequence
pub fn create_string() {
    let args = cli_app::parse_args();
    let image = args.image();
    let now = std::time::Instant::now();
    let height = image.height();
    let width = image.width();
    let mut ref_image = RefImage::new(width, height);
    image
        .to_luma8()
        .enumerate_pixels()
        .for_each(|(x, y, p)| ref_image[(x, y)] = p[0].into());

    let initial_score = ref_image.score();
    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    if args.verbosity > 2 {
        ref_image
            .grayscale()
            .save("initial_reference_image.png")
            .unwrap();
    }

    let pins = match &args.pin_arrangement[..] {
        "perimeter" => generate_pins::perimeter(args.pin_count, width, height),
        "grid" => generate_pins::grid(args.pin_count, width, height),
        "circle" => generate_pins::circle(args.pin_count, width, height),
        "random" => generate_pins::random(args.pin_count, width, height),
        a => panic!("That's not a valid pin arrangement: {}", a),
    };

    if let Some(ref filepath) = args.pins_filepath {
        pins.iter()
            .fold(&mut RefImage::new(width, height), |i, p| {
                i[p] = u8::MAX as i64;
                i
            })
            .grayscale()
            .save(filepath)
            .unwrap();
    }

    let mut pin_order: Vec<(Point, Point)> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    while keep_adding || keep_removing {
        while keep_adding {
            match optimum::find_best_points(&pins, &ref_image, args.step_size, args.string_alpha) {
                Some(((a, b), s)) => {
                    // The ref_image is a hypothetical perfectly-string-drawn image, and this is
                    // trying to figure out which strings are in the image. So every time it
                    // chooses a string here, the string is removed from the ref_image.
                    ref_image.subtract_line(((a, b), args.step_size, args.string_alpha));
                    pin_order.push((a, b));
                    keep_removing = true;
                    if args.verbosity > 0 {
                        println!(
                            "[{:>6}]:   score change: {:>10}     added  {} to {}",
                            pin_order.len(),
                            s,
                            a,
                            b
                        );
                    }
                }
                None => keep_adding = false,
            }

            if pin_order.len() >= args.max_strings {
                keep_adding = false
            }
        }

        while keep_removing {
            match optimum::find_worst_points(
                &pin_order,
                &ref_image,
                args.step_size,
                args.string_alpha,
            ) {
                Some((i, s)) => {
                    let (a, b) = pin_order.remove(i);
                    // The ref_image is a hypothetical perfectly-string-drawn image, and this is
                    // trying to figure out which strings are missing from the image. So every time
                    // it chooses a string here, the string is added back into the ref_image.
                    ref_image.add_line(((a, b), args.step_size, args.string_alpha));
                    keep_adding = true;
                    if args.verbosity > 0 {
                        println!(
                            "[{:>6}]:   score change: {:>10}    removed {} to {}",
                            pin_order.len(),
                            s,
                            a,
                            b
                        );
                    }
                }
                None => keep_removing = false,
            }

            if pin_order.is_empty() {
                keep_removing = false
            }
        }
    }

    let final_score = ref_image.score();
    if args.verbosity > 1 {
        println!("(Recap) Initial score: {} (lower is better)", initial_score);
        println!("Final score          : {}", final_score);
    }
    if args.verbosity > 0 {
        println!("Saving image...");
    }

    if args.verbosity > 2 {
        ref_image
            .grayscale()
            .save("final_reference_image.png")
            .unwrap();
    }

    if let Some(ref filepath) = args.output_filepath {
        pin_order
            .iter()
            .map(|(a, b)| (*a, *b))
            .map(Line::from)
            .map(|l| (l, args.step_size, args.string_alpha))
            .fold(&mut RefImage::new(width, height), |i, a| i.add_line(a))
            .grayscale()
            .save(filepath)
            .unwrap();
    }

    if args.verbosity > 0 {
        println!("Image saved!")
    }

    if let Some(data_filepath) = args.data_filepath.clone() {
        let data = Data {
            args,
            image_height: height,
            image_width: width,
            initial_score,
            final_score,
            elapsed_seconds: now.elapsed().as_secs_f64(),
            pin_locations: pins,
            line_segments: pin_order,
        };

        std::fs::write(data_filepath, data.to_json_string()).expect("Unable to write file");
    }
}
