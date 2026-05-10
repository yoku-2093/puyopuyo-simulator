mod controller;
mod game;
mod render;
mod settings;
mod types;

use controller::Controller;
use macroquad::prelude::*;

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 900.0;

#[macroquad::main(window_conf)]
async fn main() {
    let mut controller = Controller::new(WINDOW_WIDTH, WINDOW_HEIGHT).await;

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
