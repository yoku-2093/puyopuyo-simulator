mod constants;
mod render;

use constants::*;
use macroquad::prelude::*;

#[macroquad::main(window_conf)]
async fn main() {
    let renderer = render::Renderer::new().await;

    loop {
        renderer.draw_background();

        draw_text("Hello, PuyoPuyo Simulator!", 20.0, 20.0, 30.0, BLACK);

        renderer.draw_field();
        renderer.draw_puyo(render::PuyoColor::Blue, 0, 0);
        renderer.draw_puyo(render::PuyoColor::Blue, 1, 0);
        renderer.draw_puyo(render::PuyoColor::Blue, 2, 0);
        renderer.draw_puyo(render::PuyoColor::Blue, 3, 0);
        renderer.draw_puyo(render::PuyoColor::Blue, 4, 0);
        renderer.draw_puyo(render::PuyoColor::Blue, 5, 0);
        renderer.draw_puyo(render::PuyoColor::Blue, 0, 11);
        renderer.draw_puyo(render::PuyoColor::Blue, 1, 11);
        renderer.draw_puyo(render::PuyoColor::Blue, 2, 11);
        renderer.draw_puyo(render::PuyoColor::Blue, 3, 11);
        renderer.draw_puyo(render::PuyoColor::Blue, 4, 11);
        renderer.draw_puyo(render::PuyoColor::Blue, 5, 11);

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
