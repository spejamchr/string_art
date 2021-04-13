use super::geometry::Point;
use super::imagery::RefImage;
use crate::imagery::LineSegment;
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
) -> Vec<(LineSegment, i64)> {
    let mut lines = pins
        .par_iter()
        .enumerate()
        .flat_map(|(i, a)| pins.par_iter().skip(i).map(move |b| (a, b)))
        .filter(|(a, b)| a.x != b.x && a.y != b.y)
        .flat_map(|(a, b)| {
            rgbs.par_iter().map(move |rgb| {
                (
                    (*a, *b, *rgb),
                    ref_image.score_change_if_added(((*a, *b), *rgb, step_size, string_alpha)),
                )
            })
        })
        .filter(|(_, s)| *s < 0)
        .collect::<Vec<_>>();
    lines.sort_unstable_by_key(|(_, s)| *s);
    lines.into_iter().take(max).collect()
}

pub fn find_worst_points(
    points: &[LineSegment],
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
                ref_image.score_change_if_removed(((*a, *b), *rgb, step_size, string_alpha)),
            )
        })
        .filter(|(_, s)| *s < 0)
        .collect::<Vec<_>>();
    lines.sort_unstable_by_key(|(_, s)| *s);
    lines.into_iter().take(max).collect()
}
