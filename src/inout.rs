use super::cli_app::Args;
use super::geometry::Point;
use crate::imagery::LineSegment;
use crate::imagery::RGB;

pub trait ToJsonString {
    fn to_json_string(&self) -> String;
}

impl ToJsonString for Args {
    fn to_json_string(&self) -> String {
        format!(
            r#"{{"max_strings":{},"step_size":{},"string_alpha":{},"pin_count":{},"pin_arrangement":"{}","style":"{}","auto_color_limit":{},"rgbs":[{}],"verbosity":{},"input_filepath":"{}","output_filepath":{},"pins_filepath":{},"data_filepath":{},"gif_filepath":{}}}"#,
            self.max_strings,
            self.step_size,
            self.string_alpha,
            self.pin_count,
            self.pin_arrangement,
            self.style,
            self.auto_color_limit,
            self.rgbs
                .iter()
                .map(|p| format!(r#""{}""#, p))
                .collect::<Vec<_>>()
                .join(","),
            self.verbosity,
            self.input_filepath,
            self.output_filepath.to_json_string(),
            self.pins_filepath.to_json_string(),
            self.data_filepath.to_json_string(),
            self.gif_filepath.to_json_string(),
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
    pub line_segments: Vec<LineSegment>,
}

impl ToJsonString for Data {
    fn to_json_string(&self) -> String {
        format!(
            r#"{{"args":{},"image_height":{},"image_width":{},"initial_score":{},"final_score":{},"elapsed_seconds":{},"pin_count":{},"line_count":{},"pin_locations":{},"line_segments":{}}}"#,
            self.args.to_json_string(),
            self.image_height,
            self.image_width,
            self.initial_score,
            self.final_score,
            self.elapsed_seconds,
            self.pin_locations.len(),
            self.line_segments.len(),
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
        format!(r#"[{},{}]"#, self.x, self.y)
    }
}

impl ToJsonString for RGB {
    fn to_json_string(&self) -> String {
        format!(r#""{}""#, self)
    }
}

impl ToJsonString for LineSegment {
    fn to_json_string(&self) -> String {
        let points = vec![self.0, self.1].to_json_string();
        format!(r#"{{"points":{},"rgb":"{}"}}"#, points, self.2)
    }
}
