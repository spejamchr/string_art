use crate::cli_app::AutoColor;
use crate::image::DynamicImage;
use crate::imagery::Rgb;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn fg_and_bg(auto_color: &AutoColor, image: &DynamicImage) -> (Vec<Rgb>, Rgb) {
    let background_color = auto_color
        .manual_background
        .unwrap_or_else(|| calc_bg(image, &auto_color.manual_foregrounds));

    let foreground_colors = calc_fgs(
        image,
        &auto_color.manual_foregrounds,
        &background_color,
        auto_color.auto_fg_count,
    );

    (foreground_colors, background_color)
}

fn calc_fgs(
    image: &DynamicImage,
    foreground_colors: &HashSet<Rgb>,
    background_color: &Rgb,
    limit: usize,
) -> Vec<Rgb> {
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

fn calc_bg(image: &DynamicImage, foreground_colors: &HashSet<Rgb>) -> Rgb {
    rank_colors(image)
        .into_iter()
        .filter(|(rgb, _)| !foreground_colors.contains(rgb))
        .max_by_key(|(_, c)| *c)
        .map(|(rgb, _)| rgb)
        .unwrap()
}

fn rank_colors(image: &DynamicImage) -> HashMap<Rgb, usize> {
    image_rgbs(&image.adjust_contrast(1500.0))
        .into_iter()
        .fold(HashMap::new(), |mut h, p| {
            if let Some(old) = h.insert(p, 1) {
                h.insert(p, old + 1);
            }
            h
        })
}

fn image_rgbs(image: &DynamicImage) -> Vec<Rgb> {
    image
        .adjust_contrast(1500.0)
        .to_rgb8()
        .pixels()
        .map(|p| p.0)
        .map(Rgb::from)
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    const BLUE: Rgb = Rgb { r: 0, g: 0, b: 255 };

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

    fn p(r: u32, g: u32, b: u32) -> Rgb {
        Rgb::new(r, g, b)
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
                Rgb::WHITE,
                Rgb::WHITE,
                Rgb::WHITE,
                Rgb::WHITE,
                BLUE,
                BLUE,
                BLUE,
                Rgb::BLACK,
                Rgb::BLACK
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
        let rgbs = vec![(Rgb::WHITE, 4), (BLUE, 3), (Rgb::BLACK, 2)];
        let map: HashMap<_, _> = rgbs.into_iter().collect();
        assert_eq!(map, rank_colors(&complex_img()));
    }

    #[test]
    fn test_calc_bg_all_black() {
        assert_eq!(Rgb::BLACK, calc_bg(&black_img(), &HashSet::new()));
    }

    #[test]
    fn test_calc_bg_complex() {
        assert_eq!(Rgb::WHITE, calc_bg(&complex_img(), &HashSet::new()));
    }

    fn ac(
        auto_fg_count: usize,
        manual_foregrounds: Vec<Rgb>,
        manual_background: Option<Rgb>,
    ) -> AutoColor {
        AutoColor {
            auto_fg_count,
            manual_background,
            manual_foregrounds: manual_foregrounds.into_iter().collect(),
        }
    }

    #[test]
    fn test_fg_and_bg_1_fg() {
        assert_eq!(
            (vec![BLUE], Rgb::WHITE),
            fg_and_bg(&ac(1, Vec::new(), None), &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_2_fgs() {
        assert_eq!(
            (vec![BLUE, Rgb::BLACK], Rgb::WHITE),
            fg_and_bg(&ac(2, Vec::new(), None), &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_excess_fgs() {
        assert_eq!(
            (vec![BLUE, Rgb::BLACK], Rgb::WHITE),
            fg_and_bg(&ac(20, Vec::new(), None), &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_provided_bg() {
        assert_eq!(
            (vec![Rgb::WHITE], BLUE),
            fg_and_bg(&ac(1, Vec::new(), Some(BLUE)), &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_provided_fg() {
        assert_eq!(
            (vec![Rgb::BLACK, Rgb::WHITE], BLUE),
            fg_and_bg(&ac(1, vec![Rgb::WHITE], None), &complex_img())
        );
    }

    #[test]
    fn test_fg_and_bg_provided_fg_and_bg() {
        assert_eq!(
            (vec![BLUE, Rgb::WHITE], Rgb::BLACK),
            fg_and_bg(&ac(1, vec![Rgb::WHITE], Some(Rgb::BLACK)), &complex_img())
        );
    }
}
