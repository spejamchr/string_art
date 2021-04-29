use crate::cli_app::Args;
use crate::geometry::Point;
use crate::image::{DynamicImage, RgbaImage};
use crate::imagery::LineSegment;
use crate::imagery::RefImage;
use crate::imagery::RGB;
use crate::inout::Data;
use crate::optimum;
use image::gif::GifEncoder;
use image::Frame;
use std::fs::File;
use std::time::Instant;

pub fn color_on_custom(pin_locations: Vec<Point>, args: Args, img: DynamicImage) -> Data {
    let mut ref_image: RefImage = img.into();
    let background_color = args.background_color;
    ref_image = ref_image.negated().add_rgb(background_color);
    let colors = args
        .foreground_colors
        .iter()
        .map(|rgb| *rgb - background_color)
        .collect::<Vec<_>>();

    let (mut data, frames) = run(args, ref_image, pin_locations, &colors);

    if let Some(ref filepath) = data.args.output_filepath {
        RefImage::from(&data).color().save(filepath).unwrap();
    }

    if let Some(gif_filepath) = &data.args.gif_filepath {
        save_gif(gif_filepath, frames);
    }

    data.line_segments
        .iter_mut()
        .for_each(|(_, _, rgb)| *rgb = *rgb + background_color);

    data
}

fn save_gif(gif_filepath: &str, frames: Vec<RgbaImage>) {
    let file_out = File::create(gif_filepath).unwrap();
    let mut encoder = GifEncoder::new_with_speed(file_out, 10);
    encoder.set_repeat(image::gif::Repeat::Infinite).unwrap();
    encoder
        .encode_frames(frames.into_iter().map(Frame::new))
        .unwrap();
}

fn log_added_points(args: &Args, pin_len: usize, score_change: i64, a: Point, b: Point, rgb: RGB) {
    if args.verbosity > 0 {
        let rgb = rgb + args.background_color;
        println!(
            "[{:>6}]:   score change: {:>10}     added  {} to {} with {}",
            pin_len, score_change, a, b, rgb
        );
    }
}

fn log_removed_points(
    args: &Args,
    pin_len: usize,
    score_change: i64,
    a: Point,
    b: Point,
    rgb: RGB,
) {
    if args.verbosity > 0 {
        let rgb = rgb + args.background_color;
        println!(
            "[{:>6}]:   score change: {:>10}    removed {} to {} with {}",
            pin_len, score_change, a, b, rgb
        );
    }
}

fn run(
    args: Args,
    ref_image: RefImage,
    pin_locations: Vec<Point>,
    rgbs: &[RGB],
) -> (Data, Vec<RgbaImage>) {
    let image_width = ref_image.width();
    let image_height = ref_image.height();

    let start_at = Instant::now();
    let (line_segments, initial_score, final_score, frames) =
        implementation(&args, ref_image, &pin_locations, rgbs);

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

    (data, frames)
}

fn capture_frame(
    line_segments: &[LineSegment],
    frames: &mut Vec<RgbaImage>,
    args: &Args,
    width: u32,
    height: u32,
) {
    if args.gif_filepath.is_some() {
        let img = RefImage::from((
            &line_segments
                .iter()
                .map(|(a, b, rgb)| ((*a, *b), *rgb, args.step_size, args.string_alpha))
                .collect(),
            width,
            height,
        ));
        frames.push(img.color())
    }
}

fn implementation(
    args: &Args,
    mut ref_image: RefImage,
    pin_locations: &[Point],
    rgbs: &[RGB],
) -> (Vec<LineSegment>, i64, i64, Vec<RgbaImage>) {
    let mut line_segments: Vec<LineSegment> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    let initial_score = ref_image.score();

    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    let mut cap = 100;
    let mut max_at_once = usize::min(args.max_strings / 10, cap);

    let mut frames = Vec::new();
    let width = ref_image.width();
    let height = ref_image.height();

    while keep_adding || keep_removing {
        max_at_once = usize::min(max_at_once, cap);
        cap -= 1;

        while keep_adding {
            capture_frame(&line_segments, &mut frames, &args, width, height);

            keep_adding = false;

            let points = optimum::find_best_points(
                &pin_locations,
                &ref_image,
                args.step_size,
                args.string_alpha,
                rgbs,
                usize::min(args.max_strings - line_segments.len(), max_at_once),
            );

            if !points.is_empty() {
                keep_removing = true;
                keep_adding = true;
            }

            if points.len() == max_at_once {
                max_at_once = (max_at_once as f64 * 1.1) as usize
            }

            points.into_iter().for_each(|((a, b, rgb), s)| {
                ref_image.add_line(((a, b), rgb, args.step_size, args.string_alpha));
                line_segments.push((a, b, rgb));
                log_added_points(args, line_segments.len(), s, a, b, rgb);
            });

            if line_segments.len() >= args.max_strings {
                keep_adding = false
            }
        }

        max_at_once = usize::max(1, (max_at_once as f64 * 0.9) as usize);

        while keep_removing {
            capture_frame(&line_segments, &mut frames, &args, width, height);

            keep_removing = false;

            let mut worst_points = optimum::find_worst_points(
                &line_segments,
                &ref_image,
                args.step_size,
                args.string_alpha,
                // Find these more accurately by finding fewer at once. Saves time overall by
                // preventing strings from bouncing back and forth between added and removed.
                usize::min(line_segments.len(), usize::max(1, max_at_once / 10)),
            );
            worst_points.sort_unstable_by_key(|(i, _)| *i);
            worst_points.reverse();

            if !worst_points.is_empty() {
                keep_removing = true;
                keep_adding = true;
            }

            worst_points.into_iter().for_each(|(i, s)| {
                let (a, b, rgb) = line_segments.remove(i);
                ref_image.subtract_line(((a, b), rgb, args.step_size, args.string_alpha));
                log_removed_points(args, line_segments.len(), s, a, b, rgb);
            });

            if line_segments.is_empty() {
                keep_removing = false
            }
        }
    }

    // Pause on the last frame
    (0..10).for_each(|_| capture_frame(&line_segments, &mut frames, &args, width, height));

    let final_score = ref_image.score();
    if args.verbosity > 1 {
        println!("(Recap) Initial score: {} (lower is better)", initial_score);
        println!("Final score          : {}", final_score);
    }

    (line_segments, initial_score, final_score, frames)
}
