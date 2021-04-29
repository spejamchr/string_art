use super::cli_app::Args;
use super::geometry::Point;
use crate::imagery::LineSegment;
use serde::Serialize;

#[derive(Serialize)]
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
