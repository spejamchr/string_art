use super::geometry::Point;
use super::imagery::PixLine;
use super::imagery::RefImage;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

pub fn find_best_points(
    pins: &[Point],
    ref_image: &RefImage,
    step_size: f64,
    string_alpha: f64,
) -> Option<((Point, Point), i64)> {
    pins.par_iter()
        .enumerate()
        .flat_map(|(i, a)| pins.par_iter().skip(i).map(move |b| (a, b)))
        .filter(|(a, b)| a.x != b.x && a.y != b.y)
        .map(|(a, b)| {
            (
                (*a, *b),
                line_score(((*a, *b), step_size, string_alpha), &ref_image),
            )
        })
        .filter(|(_, s)| *s < 0)
        .min_by_key(|(_, s)| *s)
}

pub fn find_worst_points(
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
        .filter(|(_, s)| *s < 0)
        .min_by_key(|(_, s)| *s)
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
