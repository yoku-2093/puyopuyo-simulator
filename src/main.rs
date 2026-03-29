mod constants;
mod game;
mod puyo;
mod render;

use constants::*;
use macroquad::prelude::*;

#[macroquad::main(window_conf)]
async fn main() {
    let renderer = render::Renderer::new().await;
    let mut game = game::Game::new();

    loop {
        clear_background(BLACK);
        draw_text("Hello, PuyoPuyo Simulator!", 20.0, 20.0, 30.0, BLACK);
        renderer.draw_background();
        renderer.draw_field();

        game.update();

        // 積まれたぷよ
        let field = game.field();
        for row in 0..ROWS {
            for col in 0..COLS {
                if let Some(color) = field[row][col] {
                    renderer.draw_puyo(color, col, row);
                }
            }
        }

        // 落下中のぷよ
        for (color, pos) in game.falling() {
            renderer.draw_puyo(color, pos.col(), pos.row());
        }

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
