use crate::cli_app::Args;
use crate::geometry::Point;
use crate::imagery::RefImageCol;
use crate::imagery::RGB;
use crate::inout::Data;
use crate::optimum;
use std::time::Instant;

fn log_added_points(
    verbosity: u64,
    pin_len: usize,
    score_change: i64,
    a: Point,
    b: Point,
    rgb: RGB,
) {
    if verbosity > 0 {
        let rgb = rgb.inverted();
        println!(
            "[{:>6}]:   score change: {:>10}     added  {} to {} with {}",
            pin_len, score_change, a, b, rgb
        );
    }
}

fn log_removed_points(
    verbosity: u64,
    pin_len: usize,
    score_change: i64,
    a: Point,
    b: Point,
    rgb: RGB,
) {
    if verbosity > 0 {
        let rgb = rgb.inverted();
        println!(
            "[{:>6}]:   score change: {:>10}    removed {} to {} with {}",
            pin_len, score_change, a, b, rgb
        );
    }
}

pub fn run(
    args: Args,
    ref_image: &mut RefImageCol,
    pin_locations: Vec<Point>,
    rgbs: &[RGB],
) -> Data {
    let image_width = ref_image.width();
    let image_height = ref_image.height();
    ref_image.color().save("/Users/spencer/before.png").unwrap();

    let start_at = Instant::now();
    let (line_segments, initial_score, final_score) =
        implementation(&args, ref_image, &pin_locations, rgbs);

    let elapsed_seconds = start_at.elapsed().as_secs_f64();

    Data {
        args,
        image_height,
        image_width,
        initial_score,
        final_score,
        elapsed_seconds,
        pin_locations,
        line_segments,
    }
}

fn implementation(
    args: &Args,
    ref_image: &mut RefImageCol,
    pin_locations: &[Point],
    rgbs: &[RGB],
) -> (Vec<(Point, Point, RGB)>, i64, i64) {
    let mut line_segments: Vec<(Point, Point, RGB)> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    let initial_score = ref_image.score();

    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    let mut max_at_once = usize::min(args.max_strings / 10, 100);

    while keep_adding || keep_removing {
        while keep_adding {
            keep_adding = false;

            let points = optimum::find_best_points_col(
                &pin_locations,
                &ref_image,
                args.step_size,
                args.string_alpha,
                rgbs,
                usize::min(args.max_strings - line_segments.len(), max_at_once),
            );

            if points.len() > 0 {
                keep_removing = true;
                keep_adding = true;
            }

            points.into_iter().for_each(|((a, b, rgb), s)| {
                ref_image.add_line(((a, b), rgb, args.step_size, args.string_alpha));
                line_segments.push((a, b, rgb));
                log_added_points(args.verbosity, line_segments.len(), s, a, b, rgb);
            });

            if line_segments.len() >= args.max_strings {
                keep_adding = false
            }
        }

        while keep_removing {
            keep_removing = false;

            let mut worst_points = optimum::find_worst_points_col(
                &line_segments,
                &ref_image,
                args.step_size,
                args.string_alpha,
                usize::min(line_segments.len(), max_at_once),
            );
            worst_points.sort_unstable_by_key(|(i, _)| *i);
            worst_points.reverse();

            if worst_points.len() > 0 {
                keep_removing = true;
                keep_adding = true;
            }

            worst_points.into_iter().for_each(|(i, s)| {
                let (a, b, rgb) = line_segments.remove(i);
                ref_image.subtract_line(((a, b), rgb, args.step_size, args.string_alpha));
                keep_removing = true;
                keep_adding = true;
                log_removed_points(args.verbosity, line_segments.len(), s, a, b, rgb);
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
    ref_image.color().save("/Users/spencer/after.png").unwrap();

    (line_segments, initial_score, final_score)
}
