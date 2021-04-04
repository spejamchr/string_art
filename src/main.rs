extern crate image;
extern crate rand;
extern crate threadpool;

mod cli_app;
mod generate_pins;
mod geometry;
mod imagery;
mod inout;
mod optimum;
mod string_art;
mod style;

fn main() {
    string_art::create_string();
}
