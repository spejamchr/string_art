extern crate clap;
extern crate image;
extern crate rand;
extern crate rayon;
extern crate serde;
extern crate threadpool;

mod auto_color;
mod cli_app;
mod geometry;
mod imagery;
mod optimum;
mod pins;
mod string_art;
mod style;
mod util;

fn main() {
    string_art::create_string();
}
