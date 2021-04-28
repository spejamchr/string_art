use crate::imagery::RGB;
use image::DynamicImage;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn fg_and_bg(
    num: usize,
    manual_foregrounds: &HashSet<RGB>,
    manual_background: Option<RGB>,
    image: &DynamicImage,
) -> (Vec<RGB>, RGB) {
    let background_color = manual_background.unwrap_or_else(|| calc_bg(image));
    let foreground_colors = calc_fgs(image, manual_foregrounds, &background_color, num);

    (foreground_colors, background_color)
}

fn calc_fgs(
    image: &DynamicImage,
    foreground_colors: &HashSet<RGB>,
    background_color: &RGB,
    limit: usize,
) -> Vec<RGB> {
    let mut rgbs = rank_colors(image).into_iter().collect::<Vec<_>>();
    rgbs.sort_unstable_by_key(|(_, c)| *c);
    rgbs.reverse();
    rgbs.into_iter()
        .map(|(rgb, _)| rgb)
        .filter(|rgb| !foreground_colors.contains(rgb))
        .filter(|rgb| rgb != background_color)
        .take(limit)
        .chain(foreground_colors.iter().copied())
        .collect()
}

fn calc_bg(image: &DynamicImage) -> RGB {
    rank_colors(image)
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(rgb, _)| rgb)
        .unwrap()
}

fn rank_colors(image: &DynamicImage) -> HashMap<RGB, usize> {
    image_rgbs(&image.adjust_contrast(1500.0))
        .into_iter()
        .fold(HashMap::new(), |mut h, p| {
            if let Some(old) = h.insert(p, 1) {
                h.insert(p, old + 1);
            }
            h
        })
}

fn image_rgbs(image: &DynamicImage) -> Vec<RGB> {
    image
        .adjust_contrast(1500.0)
        .to_rgb8()
        .pixels()
        .map(|p| p.0)
        .map(RGB::from)
        .collect()
}
