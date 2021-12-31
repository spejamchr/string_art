use crate::cli_app::Args;
use crate::geometry::Point;
use crate::image::gif::GifEncoder;
use crate::image::DynamicImage;
use crate::image::Frame;
use crate::imagery::LineSegment;
use crate::imagery::RefImage;
use crate::imagery::Rgb;
use crate::optimum;
use crate::serde::Serialize;
use std::fs::File;
use std::time::Instant;

#[derive(Serialize)]
pub struct Data {
    pub args: Args,
    pub image_height: u32,
    pub image_width: u32,
    pub initial_score: i64,
    pub final_score: i64,
    pub elapsed_seconds: f64,
    pub pin_locations: Vec<Point>,
    pub line_segments: Vec<LineSegment>,
}

pub fn color_on_custom(pin_locations: Vec<Point>, args: Args, img: DynamicImage) -> Data {
    let background_color = args.background_color;
    let mut ref_image = RefImage::from(img).negated().add_rgb(background_color);
    let colors = args
        .foreground_colors
        .iter()
        .map(|rgb| *rgb - background_color)
        .collect::<Vec<_>>();

    let start_at = Instant::now();
    let (line_segments, initial_score, final_score) =
        implementation(&args, &mut ref_image, &pin_locations, &colors);

    let data = Data {
        args,
        image_height: ref_image.height(),
        image_width: ref_image.width(),
        initial_score,
        final_score,
        elapsed_seconds: start_at.elapsed().as_secs_f64(),
        pin_locations,
        line_segments: line_segments
            .into_iter()
            .map(|(a, b, rgb)| (a, b, rgb + background_color))
            .collect(),
    };

    if let Some(ref filepath) = data.args.output_filepath {
        RefImage::from(&data).color().save(filepath).unwrap();
    }

    data
}

fn log_on_add(args: &Args, pin_len: usize, score_change: i64, a: Point, b: Point, rgb: Rgb) {
    if args.verbosity > 0 {
        let rgb = rgb + args.background_color;
        println!(
            "[{:>6}]:   score change: {:>10}     +add  {} to {} with {}",
            pin_len, score_change, a, b, rgb
        );
    }
}

fn log_on_sub(args: &Args, pin_len: usize, score_change: i64, a: Point, b: Point, rgb: Rgb) {
    if args.verbosity > 0 {
        let rgb = rgb + args.background_color;
        println!(
            "[{:>6}]:   score change: {:>10}     -sub  {} to {} with {}",
            pin_len, score_change, a, b, rgb
        );
    }
}

fn capture_frame(
    possible_encoder: &mut Option<GifEncoder<File>>,
    line_segments: &[LineSegment],
    args: &Args,
    width: u32,
    height: u32,
) {
    if let Some(encoder) = possible_encoder {
        let lines = line_segments
            .iter()
            .map(|(a, b, rgb)| ((*a, *b), *rgb, args.step_size, args.string_alpha))
            .collect();
        let img = RefImage::from((&lines, width, height)).color();
        encoder.encode_frame(Frame::new(img)).unwrap();
    }
}

fn implementation(
    args: &Args,
    ref_image: &mut RefImage,
    pin_locations: &[Point],
    rgbs: &[Rgb],
) -> (Vec<LineSegment>, i64, i64) {
    let mut line_segments: Vec<LineSegment> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    let initial_score = ref_image.score();

    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    let mut cap = 100;
    let mut max_at_once = usize::min(args.max_strings / 10, cap);

    let mut possible_encoder: Option<GifEncoder<File>> =
        args.gif_filepath.as_ref().map(|gif_filepath| {
            let file_out = File::create(gif_filepath).unwrap();
            let mut encoder = GifEncoder::new_with_speed(file_out, 10);
            encoder.set_repeat(image::gif::Repeat::Infinite).unwrap();
            encoder
        });

    let width = ref_image.width();
    let height = ref_image.height();

    while keep_adding || keep_removing {
        max_at_once = usize::min(max_at_once, cap);
        cap -= 1;

        while keep_adding {
            capture_frame(&mut possible_encoder, &line_segments, args, width, height);

            keep_adding = false;

            let points = optimum::find_best_points(
                pin_locations,
                ref_image,
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
                *ref_image += ((a, b), rgb, args.step_size, args.string_alpha);
                line_segments.push((a, b, rgb));
                log_on_add(args, line_segments.len(), s, a, b, rgb);
            });

            if line_segments.len() >= args.max_strings {
                keep_adding = false
            }
        }

        max_at_once = usize::max(1, (max_at_once as f64 * 0.9) as usize);

        while keep_removing {
            capture_frame(&mut possible_encoder, &line_segments, args, width, height);

            keep_removing = false;

            let mut worst_points = optimum::find_worst_points(
                &line_segments,
                ref_image,
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
                *ref_image -= ((a, b), rgb, args.step_size, args.string_alpha);
                log_on_sub(args, line_segments.len(), s, a, b, rgb);
            });

            if line_segments.is_empty() {
                keep_removing = false
            }
        }
    }

    // Pause on the last frame
    (0..10).for_each(|_| capture_frame(&mut possible_encoder, &line_segments, args, width, height));

    let final_score = ref_image.score();
    if args.verbosity > 1 {
        println!("(Recap) Initial score: {} (lower is better)", initial_score);
        println!("Final score          : {}", final_score);
    }

    (line_segments, initial_score, final_score)
}
