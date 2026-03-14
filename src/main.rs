use macroquad::prelude::*;


#[macroquad::main("PuyoPuyo Simulator")]
async fn main() {
    loop {
        clear_background(WHITE);

        draw_text("Hello, PuyoPuyo Simulator!", 20.0, 20.0, 30.0, BLACK);

        next_frame().await;
    }
}