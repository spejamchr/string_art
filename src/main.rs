extern crate image;
extern crate rand;
extern crate threadpool;

use crate::rand::RngCore;
use clap::{App, Arg};
use image::io::Reader as ImageReader;
use image::GenericImageView;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("strings")
        .version("0.1.0")
        .about("Transform an image into string art")
        .arg(
            Arg::with_name("image_filepath")
                .value_name("IMAGE_FILEPATH")
                .takes_value(true)
                .required(true)
                .help("Path to the image that will be rendered with strings"),
        )
        .arg(
            Arg::with_name("output_filepath")
                .value_name("FILEPATH")
                .short("o")
                .long("output-filepath")
                .takes_value(true)
                .help("Location to save generated string image"),
        )
        .arg(
            Arg::with_name("pins_filepath")
                .value_name("FILEPATH")
                .short("p")
                .long("pins-filepath")
                .takes_value(true)
                .help("Location to save image of pin locations"),
        )
        .arg(
            Arg::with_name("data_filepath")
                .value_name("FILEPATH")
                .short("d")
                .long("data-filepath")
                .takes_value(true)
                .help("Location for ouput of JSON with operation information that includes: argument values, starting and ending image scores, pin locations, and a list of line segments between pins that form the final image")
        )
        .arg(
            Arg::with_name("max_strings")
                .value_name("INTEGER")
                .short("m")
                .long("max-strings")
                .takes_value(true)
                .validator(|f| {
                    f.parse::<usize>()
                        .map(|_| ())
                        .map_err(|e| format!("{:?}", e))
                })
                .help("The maximum number of strings in the finished work"),
        )
        .arg(
            Arg::with_name("step_size")
                .value_name("FLOAT")
                .short("s")
                .long("step-size")
                .takes_value(true)
                .default_value("1")
                .validator(|f| {
                        f.parse::<f64>()
                            .map(|_| ())
                            .map_err(|e| format!("{:?}", e))
                })
                .help("Used when calculating a string's antialiasing. Smaller values -> finer antialiasing")
        )
        .arg(
            Arg::with_name("string_alpha")
                .value_name("FLOAT")
                .short("a")
                .long("string-alpha")
                .takes_value(true)
                .default_value("1")
                .validator(|f| {
                        f.parse::<f64>()
                            .map_err(|e| format!("{:?}", e))
                            .and_then(|i|
                                if i > 0.0 && i <= 1.0 {
                                    Ok(i)
                                } else {
                                    Err(format!("{} is outside the range (0, 1]", i))
                                }
                            )
                            .map(|_| ())
                })
                .help("How opaque each string is: 1 is entirely opaque.")
        )
        .arg(
            Arg::with_name("pin_count")
                .value_name("INTEGER")
                .short("c")
                .long("pin-count")
                .takes_value(true)
                .default_value("200")
                .validator(|f| {
                        f.parse::<u32>()
                            .map(|_| ())
                            .map_err(|e| format!("{:?}", e))
                })
                .help("How many pins should be used in creating the image (approximately)")
        )
        .arg(
            Arg::with_name("pin_arrangement")
                .value_name("ARRANGEMENT")
                .short("r")
                .long("pin-arrangement")
                .takes_value(true)
                .possible_values(&["perimeter", "grid", "circle", "random"])
                .default_value("perimeter")
                .help("Should the pins be arranged on the image's perimeter, or in a grid across the entire image, or in the largest possible centered circle, or scattered randomly?")
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Output debugging messages")
        )
        .get_matches();

    let image_filepath = matches
        .value_of("image_filepath")
        .expect("The image_filepath is a required arg");

    let output_filepath = matches.value_of("output_filepath");

    let pins_filepath = matches.value_of("pins_filepath");

    let data_filepath = matches.value_of("data_filepath");

    let max_strings = matches
        .value_of("max_strings")
        .unwrap_or(&format!("{}", usize::MAX))
        .parse::<usize>()
        .expect("This should have passed validation already");

    let step_size = matches
        .value_of("step_size")
        .expect("There is a default")
        .parse::<f64>()
        .expect("This should have passed validation already");

    let string_alpha = matches
        .value_of("string_alpha")
        .expect("There is a default")
        .parse::<f64>()
        .expect("This should have passed validation already");

    let pin_count = matches
        .value_of("pin_count")
        .expect("There is a default")
        .parse::<u32>()
        .expect("This should have passed validation already");

    let pin_arrangement = matches
        .value_of("pin_arrangement")
        .expect("There is a default");

    let verbosity = matches.occurrences_of("verbose");

    if verbosity > 1 {
        if let Some(filepath) = output_filepath {
            println!("Received output_filepath: {}", filepath);
        }
        if let Some(filepath) = pins_filepath {
            println!("Received pins_filepath: {}", filepath);
        }
        if let Some(filepath) = data_filepath {
            println!("Received data_filepath: {}", filepath);
        }
        if matches.is_present("max_strings") {
            println!("Received max_strings: {}", max_strings);
        }
        if matches.is_present("step_size") {
            println!("Received step_size: {}", step_size);
        }
        if matches.is_present("string_alpha") {
            println!("Received string_alpha: {}", string_alpha);
        }
        if matches.is_present("pin_count") {
            println!("Received pin_count: {}", pin_count);
        }
        if matches.is_present("pin_arrangement") {
            println!("Received pin_arrangement: {}", pin_arrangement);
        }
        if matches.is_present("verbose") {
            println!("Received verbose: {}", verbosity);
        }
    }

    let image = ImageReader::open(image_filepath)?
        .decode()
        .expect("Corrupted file");

    let args = Args {
        max_strings,
        step_size,
        string_alpha,
        pin_count,
        pin_arrangement,
        output_filepath,
        pins_filepath,
        verbosity,
    };

    let data = create_string(image, args);

    if let Some(filepath) = data_filepath {
        std::fs::write(filepath, data.to_json_string()).expect("Unable to write file");
    }

    Ok(())
}

trait ToJsonString {
    fn to_json_string(&self) -> String;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn basis(&self) -> Self {
        *self / self.len()
    }

    pub fn dist(&self, other: &Self) -> f64 {
        (*other - *self).len()
    }
}

impl std::ops::Add for Vector {
    type Output = Self;
    fn add(self, rhs: Self) -> <Self as std::ops::Add>::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vector {
    type Output = Self;
    fn sub(self, rhs: Self) -> <Self as std::ops::Add>::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Vector {
    type Output = Self;
    fn mul(self, num: f64) -> <Self as std::ops::Mul<f64>>::Output {
        Self::new(self.x * num, self.y * num)
    }
}

impl std::ops::Div<f64> for Vector {
    type Output = Self;
    fn div(self, num: f64) -> <Self as std::ops::Div<f64>>::Output {
        Self::new(self.x / num, self.y / num)
    }
}

impl std::convert::From<Point> for Vector {
    fn from(point: Point) -> Self {
        Self::new(point.x as f64, point.y as f64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Line(Vector, Vector);

impl Line {
    pub fn iter(&self, step_size: f64) -> LineIter {
        let basis = (self.1 - self.0).basis();
        let current = self.0;
        let distance = self.0.dist(&self.1);

        LineIter {
            basis,
            current,
            distance,
            step_size,
        }
    }
}

impl<T: Into<Vector>> std::convert::From<(T, T)> for Line {
    fn from((a, b): (T, T)) -> Self {
        Self(a.into(), b.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LineIter {
    basis: Vector,
    current: Vector,
    distance: f64,
    step_size: f64,
}

impl Iterator for LineIter {
    type Item = Vector;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        if self.distance >= 0.0 {
            let current = self.current;
            self.current = self.current + self.basis * self.step_size;
            self.distance -= self.step_size;
            Some(current)
        } else {
            None
        }
    }
}

#[test]
fn test_line_iter() {
    let line = Line(Vector::new(0.0, 0.0), Vector::new(3.0, 4.0));
    let iter = line.iter(1.0);
    assert_eq!(6, iter.count());
}

struct PixLine(HashMap<Point, u8>);

impl PixLine {
    pub fn iter(&self) -> std::collections::hash_map::Iter<Point, u8> {
        self.0.iter()
    }
}

impl<T: Into<Line>> std::convert::From<(T, f64, f64)> for PixLine {
    fn from((line, step_size, string_alpha): (T, f64, f64)) -> Self {
        let coloring_val = step_size * string_alpha;
        Self(
            line.into()
                .iter(step_size)
                .map(Point::from)
                .fold(HashMap::new(), |mut a, p| {
                    if let Some(old) = a.insert(p, coloring_val) {
                        a.insert(p, old + coloring_val);
                    }
                    a
                })
                .into_iter()
                .map(|(p, v)| (p, (f64::min(1.0, v) * (u8::MAX as f64)) as u8))
                .collect::<HashMap<_, _>>(),
        )
    }
}

#[test]
fn test_pix_line_iter() {
    let pix_line = PixLine::from((Line(Vector::new(0.0, 0.0), Vector::new(4.0, 1.0)), 0.1, 1.0));
    assert_eq!(6, pix_line.iter().count());
}

struct RefImage(Vec<Vec<i64>>);

impl RefImage {
    fn new(width: u32, height: u32) -> Self {
        Self(vec![vec![0; width as usize]; height as usize])
    }

    fn score(&self) -> i64 {
        self.0.iter().flatten().map(|p| p * p).sum() // Seems to be worse?
    }

    fn subtract_line<T: Into<PixLine>>(&mut self, line: T) -> &mut Self {
        *self -= line;
        self
    }

    fn add_line<T: Into<PixLine>>(&mut self, line: T) -> &mut Self {
        *self += line;
        self
    }

    fn width(&self) -> u32 {
        self.0[0].len() as u32
    }

    fn height(&self) -> u32 {
        self.0.len() as u32
    }

    fn grayscale(&self) -> image::GrayImage {
        let mut img = image::GrayImage::new(self.width(), self.height());
        for (y, row) in self.0.iter().enumerate() {
            for (x, p) in row.iter().enumerate() {
                img.get_pixel_mut(x as u32, y as u32)[0] =
                    i64::max(0, i64::min(u8::MAX as i64, *p)) as u8;
            }
        }
        img
    }
}

impl<T: Into<PixLine>> std::ops::AddAssign<T> for RefImage {
    fn add_assign(&mut self, pix_line: T) {
        pix_line
            .into()
            .iter()
            .for_each(|(p, n)| self[p] += *n as i64);
    }
}

impl<T: Into<PixLine>> std::ops::SubAssign<T> for RefImage {
    fn sub_assign(&mut self, pix_line: T) {
        pix_line
            .into()
            .iter()
            .for_each(|(p, n)| self[p] -= *n as i64);
    }
}

impl std::ops::Index<&Point> for RefImage {
    type Output = i64;
    fn index(&self, point: &Point) -> &Self::Output {
        &self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::Index<(u32, u32)> for RefImage {
    type Output = i64;
    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        &self.0[y as usize][x as usize]
    }
}

impl std::ops::IndexMut<&Point> for RefImage {
    fn index_mut(&mut self, point: &Point) -> &mut Self::Output {
        &mut self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::IndexMut<(u32, u32)> for RefImage {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Self::Output {
        &mut self.0[y as usize][x as usize]
    }
}

fn find_best_points(
    pins: &[Point],
    ref_image: &RefImage,
    step_size: f64,
    string_alpha: f64,
) -> Option<((Point, Point), i64)> {
    pins.par_iter()
        .filter_map(|a| {
            pins.iter()
                .filter(move |b| a.x != b.x && a.y != b.y)
                .map(move |b| (a, b))
                .map(|(a, b)| {
                    (
                        (*a, *b),
                        line_score(((*a, *b), step_size, string_alpha), &ref_image),
                    )
                })
                .filter(|(_, s)| s < &0)
                .min_by_key(|(_, s)| *s)
        })
        .min_by_key(|(_, s)| *s)
}

fn find_worst_points(
    points: &[(Point, Point)],
    ref_image: &RefImage,
    step_size: f64,
    string_alpha: f64,
) -> Option<(usize, i64)> {
    points
        .par_iter()
        .enumerate()
        .map(|(i, (a, b))| {
            (
                i,
                line_removal_score(((*a, *b), step_size, string_alpha), &ref_image),
            )
        })
        .filter(|(_, s)| s < &0)
        .min_by_key(|(_, s)| *s)
}

struct Args<'a> {
    max_strings: usize,
    step_size: f64,
    string_alpha: f64,
    pin_count: u32,
    pin_arrangement: &'a str,
    output_filepath: Option<&'a str>,
    pins_filepath: Option<&'a str>,
    verbosity: u64,
}

impl ToJsonString for Args<'_> {
    fn to_json_string(&self) -> String {
        format!(
            "{{\"max_strings\":{},\"step_size\":{},\"string_alpha\":{},\"pin_count\":{},\"pin_arrangement\":\"{}\",\"output_filepath\":{},\"pins_filepath\":{},\"verbosity\":{}}}",
            self.max_strings,
            self.step_size,
            self.string_alpha,
            self.pin_count,
            self.pin_arrangement,
            self.output_filepath.to_json_string(),
            self.pins_filepath.to_json_string(),
            self.verbosity,
        )
    }
}

impl ToJsonString for Option<&str> {
    fn to_json_string(&self) -> String {
        match self {
            Some(s) => format!("{{\"kind\":\"some\",\"val\":\"{}\"}}", s),
            None => format!("{{\"kind\":\"none\"}}"),
        }
    }
}

struct Data<'a> {
    args: Args<'a>,
    image_height: u32,
    image_width: u32,
    initial_score: i64,
    final_score: i64,
    elapsed_seconds: f64,
    pin_locations: Vec<Point>,
    line_segments: Vec<(Point, Point)>,
}

impl ToJsonString for Data<'_> {
    fn to_json_string(&self) -> String {
        format!(
            "{{\"args\":{},\"image_height\":{},\"image_width\":{},\"initial_score\":{},\"final_score\":{},\"elapsed_seconds\":{},\"pin_locations\":{},\"line_segments\":{}}}",
            self.args.to_json_string(),
            self.image_height,
            self.image_width,
            self.initial_score,
            self.final_score,
            self.elapsed_seconds,
            self.pin_locations.to_json_string(),
            self.line_segments.to_json_string(),
        )
    }
}

impl<T: ToJsonString> ToJsonString for Vec<T> {
    fn to_json_string(&self) -> String {
        format!(
            "[{}]",
            self.iter()
                .map(|p| p.to_json_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl ToJsonString for Point {
    fn to_json_string(&self) -> String {
        format!("{{\"x\":{},\"y\":{}}}", self.x, self.y)
    }
}

impl ToJsonString for (Point, Point) {
    fn to_json_string(&self) -> String {
        vec![self.0, self.1].to_json_string()
    }
}

// Create an image of the string art and output the knob positions and sequence
fn create_string(image: image::DynamicImage, args: Args) -> Data {
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
        println!("Initial score: {}", initial_score);
    }

    if args.verbosity > 2 {
        ref_image
            .grayscale()
            .save("initial_reference_image.png")
            .unwrap();
    }

    let pins = match args.pin_arrangement {
        "perimeter" => generate_pins_perimeter(args.pin_count, width, height),
        "grid" => generate_pins_grid(args.pin_count, width, height),
        "circle" => generate_pins_circle(args.pin_count, width, height),
        "random" => generate_pins_random(args.pin_count, width, height),
        a => panic!("That's not a valid pin arrangement: {}", a),
    };

    if let Some(filepath) = args.pins_filepath {
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
            match find_best_points(&pins, &ref_image, args.step_size, args.string_alpha) {
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

            if pin_order.len() > args.max_strings {
                keep_adding = false
            }
        }

        while keep_removing {
            match find_worst_points(&pin_order, &ref_image, args.step_size, args.string_alpha) {
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
        println!("Final score  : {}", final_score);
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

    if let Some(filepath) = args.output_filepath {
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

    Data {
        args,
        image_height: height,
        image_width: width,
        initial_score,
        final_score,
        elapsed_seconds: now.elapsed().as_secs_f64(),
        pin_locations: pins,
        line_segments: pin_order,
    }
}

/// The change in a RefImage's score when adding a Line
fn line_score<T: Into<PixLine>>(line: T, image: &RefImage) -> i64 {
    line.into()
        .iter()
        .map(|(p, v)| {
            let before = image[p];
            let after = before - *v as i64;
            after * after - before * before
        })
        .sum::<i64>()
}

/// The change in a RefImage's score when removing a Line
fn line_removal_score<T: Into<PixLine>>(line: T, image: &RefImage) -> i64 {
    line.into()
        .iter()
        .map(|(p, v)| {
            let before = image[p];
            let after = before + *v as i64;
            after * after - before * before
        })
        .sum::<i64>()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    x: u32,
    y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    // pub fn taxicab_distance(&self, other: &Point) -> u32 {
    //     ((self.x as i64 - other.x as i64).abs() + (self.y as i64 - other.y as i64).abs()) as u32
    // }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "({:>6}, {:>6})", self.x, self.y)
    }
}

impl std::convert::From<Vector> for Point {
    fn from(vector: Vector) -> Self {
        Self::new(vector.x.round() as u32, vector.y.round() as u32)
    }
}

fn generate_pins_grid(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let r = width as f64 / height as f64;
    let x = (desired_count as f64 * r).sqrt().round() as usize;
    let y = (desired_count as f64 / r).sqrt().round() as usize;
    let dx = width as f64 / x as f64;
    let dy = height as f64 / y as f64;
    let max_x = width - 1;
    let max_y = height - 1;

    (0..=y)
        .flat_map(|j| {
            (0..=x).map(move |i| {
                Point::new(
                    u32::min(max_x, (i as f64 * dx) as u32),
                    u32::min(max_y, (j as f64 * dy) as u32),
                )
            })
        })
        .collect()
}

fn generate_pins_random(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let mut rng = rand::thread_rng();
    (0..desired_count)
        .map(|_| (rng.next_u32() % width, rng.next_u32() % height))
        .map(|(x, y)| Point::new(x, y))
        .collect()
}

fn generate_pins_circle(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let center_x = (width - 1) as f64 / 2.0;
    let center_y = (height - 1) as f64 / 2.0;
    let radius = f64::min(center_x, center_y);
    let step_size = std::f64::consts::PI * 2.0 / desired_count as f64;
    (0..desired_count)
        .map(|step| {
            Point::new(
                ((radius * (step as f64 * step_size).cos()).round() + center_x) as u32,
                ((radius * (step as f64 * step_size).sin()).round() + center_y) as u32,
            )
        })
        .collect()
}

fn generate_pins_perimeter(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let desired_count = u32::max(4, desired_count);
    let spacingf = f64::max(
        1.0,
        ((width + height - 2) * 2) as f64 / desired_count as f64,
    );
    let countf = ((width + height - 2) * 2) as f64 / spacingf;
    let ratio = width as f64 / height as f64;
    let h_countf = countf / 2.0 * ratio / (1.0 + ratio);
    let v_countf = countf / 2.0 - h_countf;

    let horizontal_count = h_countf.round() as u32;
    let vertical_count = v_countf.round() as u32;
    let h_spacingf = (width as f64) / (horizontal_count as f64);
    let v_spacingf = (height as f64) / (vertical_count as f64);

    let top = (0..horizontal_count).map(|i| Point::new(f_mul(i, h_spacingf), 0));
    let bottom =
        (0..horizontal_count).map(|i| Point::new(width - f_mul(i, h_spacingf) - 1, height - 1));
    let left = (0..vertical_count).map(|i| Point::new(0, height - f_mul(i, v_spacingf) - 1));
    let right = (0..vertical_count).map(|i| Point::new(width - 1, f_mul(i, v_spacingf)));

    top.chain(right).chain(bottom).chain(left).collect()
}

#[test]
fn test_specifying_too_few_pins_returns_minimum() {
    let pins = generate_pins_perimeter(0, 1234, 1234);
    assert_eq!(4, pins.len())
}

#[test]
fn test_specifying_too_many_pins_returns_maximum() {
    let pins = generate_pins_perimeter(60, 10, 10);
    assert_eq!(36, pins.len())
}

#[test]
fn test_generate_pins() {
    for count in [4, 8, 16, 60, 120, 200, 400, 1000].iter() {
        for (width, height) in [(123, 457), (2880, 1800), (1234, 5678), (10, 10000)].iter() {
            let pins = generate_pins_perimeter(*count, *width, *height);
            assert_eq!(
                *count,
                pins.len() as u32,
                "failed on count: {}, width: {}, height: {}",
                count,
                width,
                height
            );
        }
    }
}

fn f_mul(i: u32, f: f64) -> u32 {
    (i as f64 * f) as u32
}
