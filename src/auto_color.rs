use crate::imagery::RGB;
use image::DynamicImage;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn fg_and_bg(
    fgs: usize,
    manual_foregrounds: &HashSet<RGB>,
    manual_background: Option<RGB>,
    image: &DynamicImage,
) -> (Vec<RGB>, RGB) {
    let background_color =
        manual_background.unwrap_or_else(|| calc_bg(image, &manual_foregrounds));
    let foreground_colors = calc_fgs(image, manual_foregrounds, &background_color, fgs);

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

fn calc_bg(image: &DynamicImage, foreground_colors: &HashSet<RGB>) -> RGB {
    rank_colors(image)
        .into_iter()
        .filter(|(rgb, _)| !foreground_colors.contains(rgb))
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

#[cfg(test)]
mod test {
    use super::*;

    const BLUE: RGB = RGB { r: 0, g: 0, b: 255 };

    fn img() -> DynamicImage {
        let mut i = DynamicImage::new_rgb8(2, 2).to_rgb8();
        i[(0, 0)] = image::Rgb([127, 127, 127]);
        i[(1, 0)] = image::Rgb([127, 128, 127]);
        i[(0, 1)] = image::Rgb([128, 127, 127]);
        i[(1, 1)] = image::Rgb([128, 128, 127]);
        image::DynamicImage::ImageRgb8(i)
    }

    fn black_img() -> DynamicImage {
        DynamicImage::new_rgb8(2, 2)
    }

    fn complex_img() -> DynamicImage {
        let mut i = DynamicImage::new_rgb8(3, 3).to_rgb8();
        i[(0, 0)] = image::Rgb([255, 255, 255]);
        i[(1, 0)] = image::Rgb([255, 255, 255]);
        i[(2, 0)] = image::Rgb([255, 255, 255]);
        i[(0, 1)] = image::Rgb([255, 255, 255]);
        i[(1, 1)] = image::Rgb([000, 000, 255]);
        i[(2, 1)] = image::Rgb([000, 000, 255]);
        i[(0, 2)] = image::Rgb([000, 000, 255]);
        i[(1, 2)] = image::Rgb([000, 000, 000]);
        i[(2, 2)] = image::Rgb([000, 000, 000]);
        image::DynamicImage::ImageRgb8(i)
    }

    fn p(r: u32, g: u32, b: u32) -> RGB {
        RGB::new(r, g, b)
    }

    #[test]
    fn test_simple_image_rgbs() {
        assert_eq!(
            vec![p(0, 0, 0), p(0, 255, 0), p(255, 0, 0), p(255, 255, 0)],
            image_rgbs(&img())
        );
    }

    #[test]
    fn test_complex_image_rgbs() {
        assert_eq!(
            vec![
                RGB::WHITE,
                RGB::WHITE,
                RGB::WHITE,
                RGB::WHITE,
                BLUE,
                BLUE,
                BLUE,
                RGB::BLACK,
                RGB::BLACK
            ],
            image_rgbs(&complex_img())
        );
    }

    #[test]
    fn test_rank_colors_all_black() {
        let rgbs = vec![(p(0, 0, 0), 4)];
        let map: HashMap<_, _> = rgbs.into_iter().collect();
        assert_eq!(map, rank_colors(&black_img()));
    }

    #[test]
    fn test_rank_colors_all_different() {
        let rgbs = vec![
            (p(0, 0, 0), 1),
            (p(0, 255, 0), 1),
            (p(255, 0, 0), 1),
            (p(255, 255, 0), 1),
        ];
        let map: HashMap<_, _> = rgbs.into_iter().collect();
        assert_eq!(map, rank_colors(&img()));
    }

    #[test]
    fn test_rank_colors_complex() {
        let rgbs = vec![(RGB::WHITE, 4), (BLUE, 3), (RGB::BLACK, 2)];
        let map: HashMap<_, _> = rgbs.into_iter().collect();
        assert_eq!(map, rank_colors(&complex_img()));
    }

    #[test]
    fn test_calc_bg_all_black() {
        assert_eq!(RGB::BLACK, calc_bg(&black_img(), &HashSet::new()));
    }

    #[test]
    fn test_calc_bg_complex() {
        assert_eq!(RGB::WHITE, calc_bg(&complex_img(), &HashSet::new()));
    }

    #[test]
    fn test_fg_and_bg_1_fg() {
        assert_eq!(
            (vec![BLUE], RGB::WHITE),
            fg_and_bg(1, &HashSet::new(), None, &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_2_fgs() {
        assert_eq!(
            (vec![BLUE, RGB::BLACK], RGB::WHITE),
            fg_and_bg(2, &HashSet::new(), None, &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_excess_fgs() {
        assert_eq!(
            (vec![BLUE, RGB::BLACK], RGB::WHITE),
            fg_and_bg(20, &HashSet::new(), None, &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_provided_bg() {
        assert_eq!(
            (vec![RGB::WHITE], BLUE),
            fg_and_bg(1, &HashSet::new(), Some(BLUE), &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_provided_fg() {
        let fgs: HashSet<_> = vec![RGB::WHITE].into_iter().collect();
        assert_eq!(
            (vec![RGB::BLACK, RGB::WHITE], BLUE),
            fg_and_bg(1, &fgs, None, &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_provided_fg_and_bg() {
        let fgs: HashSet<_> = vec![RGB::WHITE].into_iter().collect();
        assert_eq!(
            (vec![BLUE, RGB::WHITE], RGB::BLACK),
            fg_and_bg(1, &fgs, Some(RGB::BLACK), &complex_img())
        );
    }
}
