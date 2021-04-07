use super::cli_app::Args;
use super::geometry::Point;
use crate::imagery::RGB;

pub trait ToJsonString {
    fn to_json_string(&self) -> String;
}

impl ToJsonString for Args {
    fn to_json_string(&self) -> String {
        format!(
            r#"{{"max_strings":{},"step_size":{},"string_alpha":{},"pin_count":{},"pin_arrangement":"{}","output_filepath":{},"pins_filepath":{},"verbosity":{}}}"#,
            self.max_strings,
            self.step_size,
            self.string_alpha,
            self.pin_count,
            self.pin_arrangement,
            self.output_filepath.to_json_string(),
            self.pins_filepath.to_json_string(),
            self.verbosity,
        )
    }
}

impl ToJsonString for Option<String> {
    fn to_json_string(&self) -> String {
        match self {
            Some(s) => format!(r#"{{"kind":"some","val":"{}"}}"#, s),
            None => r#"{"kind":"none"}"#.to_string(),
        }
    }
}

pub struct Data {
    pub args: Args,
    pub image_height: u32,
    pub image_width: u32,
    pub initial_score: i64,
    pub final_score: i64,
    pub elapsed_seconds: f64,
    pub pin_locations: Vec<Point>,
    pub line_segments: Vec<(Point, Point, RGB)>,
}

impl ToJsonString for Data {
    fn to_json_string(&self) -> String {
        format!(
            r#"{{"args":{},"image_height":{},"image_width":{},"initial_score":{},"final_score":{},"elapsed_seconds":{},"pin_locations":{},"line_segments":{}}}"#,
            self.args.to_json_string(),
            self.image_height,
            self.image_width,
            self.initial_score,
            self.final_score,
            self.elapsed_seconds,
            self.pin_locations.to_json_string(),
            self.line_segments.to_json_string(),
        )
    }
}

impl<T: ToJsonString> ToJsonString for Vec<T> {
    fn to_json_string(&self) -> String {
        format!(
            "[{}]",
            self.iter()
                .map(|p| p.to_json_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl ToJsonString for Point {
    fn to_json_string(&self) -> String {
        format!(r#"{{"x":{},"y":{}}}"#, self.x, self.y)
    }
}

impl ToJsonString for RGB {
    fn to_json_string(&self) -> String {
        format!(r#"{{"r":{},"g":{},"b":{}}}"#, self.r, self.g, self.b)
    }
}

impl ToJsonString for (Point, Point, RGB) {
    fn to_json_string(&self) -> String {
        let point = vec![self.0, self.1].to_json_string();
        format!(r#"{{"point":{},"rgb":{}}}"#, point, self.2.to_json_string())
    }
}
