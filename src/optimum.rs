use crate::geometry::Point;
use crate::imagery::LineSegment;
use crate::imagery::RefImage;
use crate::imagery::RGB;
use crate::rayon::iter::IndexedParallelIterator;
use crate::rayon::iter::IntoParallelRefIterator;
use crate::rayon::iter::ParallelIterator;

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
        .flat_map(|(a, b)| rgbs.par_iter().map(move |rgb| (*a, *b, *rgb)))
        .map(|(a, b, rgb)| {
            let score = ref_image.score_change_on_add(((a, b), rgb, step_size, string_alpha));
            ((a, b, rgb), score)
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
            let score = ref_image.score_change_on_sub(((*a, *b), *rgb, step_size, string_alpha));
            (i, score)
        })
        .filter(|(_, s)| *s < 0)
        .collect::<Vec<_>>();
    lines.sort_unstable_by_key(|(_, s)| *s);
    lines.into_iter().take(max).collect()
}
