use crate::cli_app::Args;
use crate::geometry::Point;
use crate::imagery::RefImageMon;
use crate::imagery::RGB;
use crate::inout::Data;
use crate::optimum;
use std::time::Instant;

pub fn black_on_white<T>(pin_locations: Vec<Point>, args: Args, imageable: T) -> Data
where
    T: Into<RefImageMon>,
{
    let mut ref_image = imageable.into();
    ref_image.invert();

    let data = run(args, &mut ref_image, pin_locations, RGB::new(0, 0, 0));

    if let Some(ref filepath) = data.args.output_filepath {
        let mut string_image = RefImageMon::from(&data);
        string_image.invert();
        string_image.grayscale().save(filepath).unwrap();
    }

    data
}

pub fn white_on_black<T>(pin_locations: Vec<Point>, args: Args, imageable: T) -> Data
where
    T: Into<RefImageMon>,
{
    let mut ref_image = imageable.into();

    let data = run(args, &mut ref_image, pin_locations, RGB::new(255, 255, 255));

    if let Some(ref filepath) = data.args.output_filepath {
        RefImageMon::from(&data).grayscale().save(filepath).unwrap();
    }

    data
}

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

fn run(args: Args, ref_image: &mut RefImageMon, pin_locations: Vec<Point>, rgb: RGB) -> Data {
    let image_width = ref_image.width();
    let image_height = ref_image.height();

    let start_at = Instant::now();
    let (line_segments, initial_score, final_score) =
        implementation(&args, ref_image, &pin_locations);

    let elapsed_seconds = start_at.elapsed().as_secs_f64();

    Data {
        args,
        image_height,
        image_width,
        initial_score,
        final_score,
        elapsed_seconds,
        pin_locations,
        line_segments: line_segments
            .into_iter()
            .map(|(a, b)| (a, b, rgb))
            .collect(),
    }
}

fn implementation(
    args: &Args,
    ref_image: &mut RefImageMon,
    pin_locations: &[Point],
) -> (Vec<(Point, Point)>, i64, i64) {
    let mut line_segments: Vec<(Point, Point)> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    let initial_score = ref_image.score();

    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    let mut max_at_once = usize::max(1, usize::min(args.max_strings / 10, 100));

    while keep_adding || keep_removing {
        while keep_adding {
            keep_adding = false;

            let points = optimum::find_best_points_mon(
                &pin_locations,
                &ref_image,
                args.step_size,
                args.string_alpha,
                max_at_once,
            );

            if points.len() > 0 {
                keep_removing = true;
                keep_adding = true;
            }

            points.into_iter().for_each(|((a, b), s)| {
                // The ref_image is a hypothetical perfectly-string-drawn image, and this is trying
                // to figure out which strings are in the image. So every time it chooses a pair of
                // points to add, the string is subtracted from the ref_image.
                ref_image.subtract_line(((a, b), args.step_size, args.string_alpha));
                line_segments.push((a, b));
                log_added_points(args.verbosity, line_segments.len(), s, a, b);
            });

            if line_segments.len() >= args.max_strings {
                keep_adding = false
            }
        }

        while keep_removing {
            keep_removing = false;

            let mut worst_points = optimum::find_worst_points_mon(
                &line_segments,
                &ref_image,
                args.step_size,
                args.string_alpha,
                max_at_once,
            );
            // Remove them from biggest index to smallest so the indices stay true.
            worst_points.sort_unstable_by_key(|(i, _)| *i);
            worst_points.reverse();

            if worst_points.len() > 0 {
                keep_removing = true;
                keep_adding = true;
            }

            worst_points.into_iter().for_each(|(i, s)| {
                let (a, b) = line_segments.remove(i);
                // The ref_image is a hypothetical perfectly-string-drawn image, and this is trying
                // to figure out which strings are missing from the image. So every time it chooses
                // a string here, the string is added back into the ref_image.
                ref_image.add_line(((a, b), args.step_size, args.string_alpha));
                log_removed_points(args.verbosity, line_segments.len(), s, a, b);
            });

            if line_segments.is_empty() {
                keep_removing = false
            }
        }

        let tenth = usize::max(1, max_at_once / 10);
        max_at_once = usize::max(1, max_at_once.saturating_sub(tenth));
    }

    let final_score = ref_image.score();
    if args.verbosity > 1 {
        println!("(Recap) Initial score: {} (lower is better)", initial_score);
        println!("Final score          : {}", final_score);
    }

    (line_segments, initial_score, final_score)
}
