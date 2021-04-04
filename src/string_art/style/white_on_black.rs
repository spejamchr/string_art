use crate::image::DynamicImage;
use crate::image::GenericImageView;
use crate::string_art::cli_app::Args;
use crate::string_art::geometry::Line;
use crate::string_art::optimum;
use crate::string_art::Data;
use crate::string_art::Point;
use crate::string_art::RefImage;
use std::time::Instant;

fn log_added_points(verbosity: u64, pin_len: usize, score_change: i64, a: Point, b: Point) {
    if verbosity > 0 {
        println!(
            "[{:>6}]:   score change: {:>10}     added  {} to {}",
            pin_len, score_change, a, b
        );
    }
}

fn log_removed_points(verbosity: u64, pin_len: usize, score_change: i64, a: Point, b: Point) {
    if verbosity > 0 {
        println!(
            "[{:>6}]:   score change: {:>10}    removed {} to {}",
            pin_len, score_change, a, b
        );
    }
}

/// Create white on black string art image and output the knob positions and sequence
pub fn run(pins: Vec<Point>, args: Args, image: DynamicImage, now: Instant) -> Data {
    // The ref_image is a hypothetical perfectly-string-drawn image, and this is trying to figure out
    // which strings are in the image. So every time we choose a pair of points to add, we end up
    // "removing" that string from the ref_image.
    let mut ref_image = RefImage::new(image.width(), image.height());
    image
        .to_luma8()
        .enumerate_pixels()
        .for_each(|(x, y, p)| ref_image[(x, y)] = p[0].into());

    let initial_score = ref_image.score();

    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    let mut pin_order: Vec<(Point, Point)> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    while keep_adding || keep_removing {
        while keep_adding {
            match optimum::find_best_points(&pins, &ref_image, args.step_size, args.string_alpha) {
                Some(((a, b), s)) => {
                    ref_image.subtract_line(((a, b), args.step_size, args.string_alpha));
                    pin_order.push((a, b));
                    keep_removing = true;
                    log_added_points(args.verbosity, pin_order.len(), s, a, b);
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
                    log_removed_points(args.verbosity, pin_order.len(), s, a, b);
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

    if let Some(ref filepath) = args.output_filepath {
        ref_image.set_all_to(0);
        pin_order
            .iter()
            .map(|(a, b)| (*a, *b))
            .map(Line::from)
            .map(|l| (l, args.step_size, args.string_alpha))
            .fold(&mut ref_image, |i, a| i.add_line(a))
            .grayscale()
            .save(filepath)
            .unwrap();
    }

    if args.verbosity > 0 {
        println!("Image saved!")
    }

    Data {
        args,
        image_height: ref_image.height(),
        image_width: ref_image.width(),
        initial_score,
        final_score,
        elapsed_seconds: now.elapsed().as_secs_f64(),
        pin_locations: pins,
        line_segments: pin_order,
    }
}
