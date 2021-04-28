extern crate image;
extern crate rand;
extern crate threadpool;

mod auto_color;
mod cli_app;
mod geometry;
mod imagery;
mod inout;
mod optimum;
mod pins;
mod string_art;
mod style;

fn main() {
    string_art::create_string();
}
