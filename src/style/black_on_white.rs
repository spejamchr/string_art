use crate::cli_app::Args;
use crate::geometry::Point;
use crate::imagery::RefImage;
use crate::inout::Data;
use crate::style::monochrome;

pub fn run<T>(pin_locations: Vec<Point>, args: Args, imageable: T) -> Data
where
    T: Into<RefImage>,
{
    let mut ref_image = imageable.into();
    ref_image.invert();

    monochrome::run(args, &mut ref_image, pin_locations, |data| {
        if let Some(ref filepath) = data.args.output_filepath {
            let mut string_image = RefImage::from(data);
            string_image.invert();
            string_image.grayscale().save(filepath).unwrap();
        }
    })
}
