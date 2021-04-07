use crate::cli_app::Args;
use crate::geometry::Point;
use crate::imagery::RefImageCol;
use crate::imagery::RGB;
use crate::inout::Data;
use crate::style::color;

pub fn run<T>(pin_locations: Vec<Point>, args: Args, imageable: T, colors: &[RGB]) -> Data
where
    T: Into<RefImageCol>,
{
    let mut ref_image = imageable.into();
    // ref_image.invert();

    let colors = colors.iter().map(|rgb| rgb.inverted()).collect::<Vec<_>>();

    let data = color::run(args, &mut ref_image, pin_locations, &colors);

    if let Some(ref filepath) = data.args.output_filepath {
        let mut image = RefImageCol::from(&data);
        image.invert();
        image.color().save(filepath).unwrap();
    }

    data
}
