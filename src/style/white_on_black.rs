use crate::cli_app::Args;
use crate::geometry::Point;
use crate::imagery::RefImageMon;
use crate::imagery::RGB;
use crate::inout::Data;
use crate::style::monochrome;

pub fn run<T>(pin_locations: Vec<Point>, args: Args, imageable: T) -> Data
where
    T: Into<RefImageMon>,
{
    let mut ref_image = imageable.into();

    let data = monochrome::run(args, &mut ref_image, pin_locations, RGB::new(255, 255, 255));

    if let Some(ref filepath) = data.args.output_filepath {
        RefImageMon::from(&data).grayscale().save(filepath).unwrap();
    }

    data
}
