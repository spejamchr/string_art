use crate::cli_app::Args;
use crate::geometry::Point;
use crate::imagery::RefImage;
use crate::inout::Data;
use crate::optimum;
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

pub fn run<T>(args: Args, ref_image: &mut RefImage, pin_locations: Vec<Point>, callback: T) -> Data
where
    T: FnOnce(&Data),
{
    let image_width = ref_image.width();
    let image_height = ref_image.height();

    let start_at = Instant::now();
    let (line_segments, initial_score, final_score) =
        implementation(&args, ref_image, &pin_locations);

    let elapsed_seconds = start_at.elapsed().as_secs_f64();

    let data = Data {
        args,
        image_height,
        image_width,
        initial_score,
        final_score,
        elapsed_seconds,
        pin_locations,
        line_segments,
    };

    callback(&data);

    data
}

fn implementation(
    args: &Args,
    ref_image: &mut RefImage,
    pin_locations: &[Point],
) -> (Vec<(Point, Point)>, i64, i64) {
    let mut line_segments: Vec<(Point, Point)> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    let initial_score = ref_image.score();

    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    while keep_adding || keep_removing {
        while keep_adding {
            match optimum::find_best_points(
                &pin_locations,
                &ref_image,
                args.step_size,
                args.string_alpha,
            ) {
                Some(((a, b), s)) => {
                    // The ref_image is a hypothetical perfectly-string-drawn image, and this is
                    // trying to figure out which strings are in the image. So every time it
                    // chooses a pair of points to add, the string is subtracted from the
                    // ref_image.
                    ref_image.subtract_line(((a, b), args.step_size, args.string_alpha));
                    line_segments.push((a, b));
                    keep_removing = true;
                    log_added_points(args.verbosity, line_segments.len(), s, a, b);
                }
                None => keep_adding = false,
            }

            if line_segments.len() >= args.max_strings {
                keep_adding = false
            }
        }

        while keep_removing {
            match optimum::find_worst_points(
                &line_segments,
                &ref_image,
                args.step_size,
                args.string_alpha,
            ) {
                Some((i, s)) => {
                    let (a, b) = line_segments.remove(i);
                    // The ref_image is a hypothetical perfectly-string-drawn image, and this is
                    // trying to figure out which strings are missing from the image. So every time
                    // it chooses a string here, the string is added back into the ref_image.
                    ref_image.add_line(((a, b), args.step_size, args.string_alpha));
                    keep_adding = true;
                    log_removed_points(args.verbosity, line_segments.len(), s, a, b);
                    if args.verbosity > 0 {
                        println!(
                            "[{:>6}]:   score change: {:>10}    removed {} to {}",
                            line_segments.len(),
                            s,
                            a,
                            b
                        );
                    }
                }
                None => keep_removing = false,
            }

            if line_segments.is_empty() {
                keep_removing = false
            }
        }
    }

    let final_score = ref_image.score();
    if args.verbosity > 1 {
        println!("(Recap) Initial score: {} (lower is better)", initial_score);
        println!("Final score          : {}", final_score);
    }

    (line_segments, initial_score, final_score)
}
