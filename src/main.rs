mod constants;
mod controller;
mod game;
mod puyo;
mod render;

use constants::*;
use controller::Controller;
use macroquad::prelude::*;

#[macroquad::main(window_conf)]
async fn main() {
    let mut controller = Controller::new().await;

    loop {
        clear_background(BLACK);

        controller.step();

        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "PuyoPuyo Simulator".to_string(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}
