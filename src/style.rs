use crate::cli_app::Args;
use crate::geometry::Point;
use crate::image::DynamicImage;
use crate::imagery::RefImage;
use crate::imagery::RGB;
use crate::inout::Data;
use crate::optimum;
use image::gif::GifEncoder;
use image::Frame;
use std::collections::HashMap;
use std::fs::File;
use std::time::Instant;

pub fn auto_color(pin_locations: Vec<Point>, mut args: Args, img: &DynamicImage) -> Data {
    let ranked_colors = rank_colors(img);
    let background = extract_background_color(&ranked_colors);
    args.rgbs = choose_rgbs(ranked_colors, &background, &args);

    if args.verbosity > 0 {
        println!(
            "Using colors {}",
            args.rgbs
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if background == RGB::white() {
        if args.verbosity > 0 {
            println!("On a white background");
        }
        color_on_white(pin_locations, args, img)
    } else {
        if args.verbosity > 0 {
            println!("On a black background");
        }
        color_on_black(pin_locations, args, img)
    }
}

fn choose_rgbs(ranked_colors: HashMap<RGB, usize>, background: &RGB, args: &Args) -> Vec<RGB> {
    let mut rgbs = ranked_colors.into_iter().collect::<Vec<_>>();
    rgbs.sort_unstable_by_key(|(_, c)| *c);
    rgbs.reverse();
    rgbs.into_iter()
        .map(|(rgb, _)| rgb)
        .filter(|rgb| rgb != background)
        .take(args.auto_color_limit as usize)
        .collect()
}

fn extract_background_color(ranking: &HashMap<RGB, usize>) -> RGB {
    let tmp_ranking = ranking.clone();
    let mut wb = tmp_ranking
        .iter()
        .filter(|(rgb, _)| rgb.r == rgb.g && rgb.g == rgb.b)
        .collect::<Vec<_>>();
    wb.sort_unstable_by_key(|(_, c)| *c);
    wb.reverse();
    let (background, _) = wb[0];
    *background
}

fn rank_colors(img: &DynamicImage) -> HashMap<RGB, usize> {
    img.adjust_contrast(1500.0)
        .to_rgb8()
        .pixels()
        .map(|p| p.0)
        .map(|[r, g, b]| RGB::new(r, g, b))
        .fold(HashMap::new(), |mut h, p| {
            if let Some(old) = h.insert(p, 1) {
                h.insert(p, old + 1);
            }
            h
        })
}

pub fn black_on_white(pin_locations: Vec<Point>, mut args: Args, img: &DynamicImage) -> Data {
    args.rgbs = vec![RGB::black()];
    color_on_white(pin_locations, args, img)
}

pub fn white_on_black(pin_locations: Vec<Point>, mut args: Args, img: &DynamicImage) -> Data {
    args.rgbs = vec![RGB::white()];
    color_on_black(pin_locations, args, img)
}

pub fn color_on_white(pin_locations: Vec<Point>, args: Args, img: &DynamicImage) -> Data {
    let colors = args
        .rgbs
        .iter()
        .map(|rgb| rgb.inverted())
        .collect::<Vec<_>>();

    let (data, frames) = run(args, &mut img.into(), pin_locations, &colors);

    if let Some(ref filepath) = data.args.output_filepath {
        RefImage::from(&data)
            .inverted()
            .color()
            .save(filepath)
            .unwrap();
    }

    if let Some(gif_filepath) = &data.args.gif_filepath {
        save_gif(gif_filepath, frames, |r| r.inverted().color());
    }

    data
}

pub fn color_on_black(pin_locations: Vec<Point>, args: Args, img: &DynamicImage) -> Data {
    let colors = args.rgbs.clone();
    let ref_image: RefImage = img.into();

    let (data, frames) = run(args, &mut ref_image.inverted(), pin_locations, &colors);

    if let Some(ref filepath) = data.args.output_filepath {
        RefImage::from(&data).color().save(filepath).unwrap();
    }

    if let Some(gif_filepath) = &data.args.gif_filepath {
        save_gif(gif_filepath, frames, |r| r.color());
    }

    data
}

fn save_gif<F>(gif_filepath: &str, frames: Vec<RefImage>, map: F)
where
    F: Fn(RefImage) -> image::RgbaImage,
{
    let file_out = File::create(gif_filepath).unwrap();
    GifEncoder::new_with_speed(file_out, 10)
        .encode_frames(frames.into_iter().map(map).map(Frame::new))
        .unwrap();
}

fn log_added_points(
    verbosity: u64,
    pin_len: usize,
    score_change: i64,
    a: Point,
    b: Point,
    rgb: RGB,
) {
    if verbosity > 0 {
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
        println!(
            "[{:>6}]:   score change: {:>10}    removed {} to {} with {}",
            pin_len, score_change, a, b, rgb
        );
    }
}

fn run(
    args: Args,
    ref_image: &mut RefImage,
    pin_locations: Vec<Point>,
    rgbs: &[RGB],
) -> (Data, Vec<RefImage>) {
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
    line_segments: &[(Point, Point, RGB)],
    frames: &mut Vec<RefImage>,
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
        frames.push(img)
    }
}

fn implementation(
    args: &Args,
    ref_image: &mut RefImage,
    pin_locations: &[Point],
    rgbs: &[RGB],
) -> (Vec<(Point, Point, RGB)>, i64, i64, Vec<RefImage>) {
    let mut line_segments: Vec<(Point, Point, RGB)> = Vec::new();
    let mut keep_adding = true;
    let mut keep_removing = true;

    let initial_score = ref_image.score();

    if args.verbosity > 1 {
        println!("Initial score: {} (lower is better)", initial_score);
    }

    let mut max_at_once = usize::min(args.max_strings / 10, 100);

    let mut frames = Vec::new();
    let width = ref_image.width();
    let height = ref_image.height();

    while keep_adding || keep_removing {
        max_at_once = usize::min(max_at_once, 100);

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
                log_added_points(args.verbosity, line_segments.len(), s, a, b, rgb);
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
                log_removed_points(args.verbosity, line_segments.len(), s, a, b, rgb);
            });

            if line_segments.is_empty() {
                keep_removing = false
            }
        }
    }

    capture_frame(&line_segments, &mut frames, &args, width, height);

    let final_score = ref_image.score();
    if args.verbosity > 1 {
        println!("(Recap) Initial score: {} (lower is better)", initial_score);
        println!("Final score          : {}", final_score);
    }

    (line_segments, initial_score, final_score, frames)
}
