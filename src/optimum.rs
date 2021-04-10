use super::geometry::Point;
use super::imagery::PixLine;
use super::imagery::RefImage;
use crate::imagery::RGB;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

pub fn find_best_points(
    pins: &[Point],
    ref_image: &RefImage,
    step_size: f64,
    string_alpha: f64,
    rgbs: &[RGB],
    max: usize,
) -> Vec<((Point, Point, RGB), i64)> {
    let mut lines = pins
        .par_iter()
        .enumerate()
        .flat_map(|(i, a)| pins.par_iter().skip(i).map(move |b| (a, b)))
        .filter(|(a, b)| a.x != b.x && a.y != b.y)
        .flat_map(|(a, b)| {
            rgbs.par_iter().map(move |rgb| {
                (
                    (*a, *b, *rgb),
                    line_score(((*a, *b), *rgb, step_size, string_alpha), &ref_image),
                )
            })
        })
        .filter(|(_, s)| *s < 0)
        .collect::<Vec<_>>();
    lines.sort_unstable_by_key(|(_, s)| *s);
    lines.into_iter().take(max).collect()
}

pub fn find_worst_points(
    points: &[(Point, Point, RGB)],
    ref_image: &RefImage,
    step_size: f64,
    string_alpha: f64,
    max: usize,
) -> Vec<(usize, i64)> {
    let mut lines = points
        .par_iter()
        .enumerate()
        .map(|(i, (a, b, rgb))| {
            (
                i,
                line_removal_score(((*a, *b), *rgb, step_size, string_alpha), &ref_image),
            )
        })
        .filter(|(_, s)| *s < 0)
        .collect::<Vec<_>>();
    lines.sort_unstable_by_key(|(_, s)| *s);
    lines.into_iter().take(max).collect()
}

/// The change in a RefImage's score when adding a Line
fn line_score<T: Into<PixLine>>(line: T, image: &RefImage) -> i64 {
    let m = u8::MAX as i64;
    line.into()
        .iter()
        .map(|(p, rgb)| {
            let a = image[p];
            let a = [a.0, a.1, a.2];
            let b = [
                a[0] + rgb.r as i64,
                a[1] + rgb.g as i64,
                a[2] + rgb.b as i64,
            ];
            let b = b.iter().map(|n| (m - n).saturating_pow(2)).sum::<i64>();
            let a = a.iter().map(|n| (m - n).saturating_pow(2)).sum::<i64>();
            b - a
        })
        .sum::<i64>()
}

/// The change in a RefImage's score when removing a Line
fn line_removal_score<T: Into<PixLine>>(line: T, image: &RefImage) -> i64 {
    let m = u8::MAX as i64;
    line.into()
        .iter()
        .map(|(p, rgb)| {
            let a = image[p];
            let a = [a.0, a.1, a.2];
            let b = [
                a[0] - rgb.r as i64,
                a[1] - rgb.g as i64,
                a[2] - rgb.b as i64,
            ];
            let b = b.iter().map(|n| (m - n).saturating_pow(2)).sum::<i64>();
            let a = a.iter().map(|n| (m - n).saturating_pow(2)).sum::<i64>();
            b - a
        })
        .sum::<i64>()
}
