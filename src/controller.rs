use crate::constants::*;
use crate::game::{GameField, GamePhase, TickResult};
use crate::puyo::Rotation;
use crate::render::Renderer;
use macroquad::prelude::*;

pub struct Controller {
    phase: GamePhase,
    renderer: Renderer,
    last_drop_time: f64,
    last_move_time: f64,
}

impl Controller {
    pub async fn new() -> Self {
        let renderer = Renderer::new().await;
        Controller {
            phase: GamePhase::new(),
            renderer,
            last_drop_time: get_time(),
            last_move_time: 0.0,
        }
    }

    pub fn step(&mut self) {
        self.renderer.draw_background();
        self.renderer.draw_field();

        match &mut self.phase {
            GamePhase::Start => {
                self.renderer.draw_title();
                self.renderer.draw_press_start();
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.phase = GamePhase::Playing(GameField::new());
                    self.last_drop_time = get_time();
                }
            }
            GamePhase::Playing(field) => {
                let now = get_time();

                // 自動落下
                if now - self.last_drop_time > DROP_INTERVAL {
                    let result = field.tick();
                    if result == TickResult::GameOver {
                        self.phase = GamePhase::GameOver(GameField::new());
                        return;
                    }
                    self.last_drop_time = now;
                }

                // 方向キー
                if is_key_down(KeyCode::Left)
                    || is_key_down(KeyCode::Right)
                    || is_key_down(KeyCode::Down)
                {
                    if now - self.last_move_time > MOVE_INTERVAL {
                        if is_key_down(KeyCode::Left) {
                            field.move_left();
                        }
                        if is_key_down(KeyCode::Right) {
                            field.move_right();
                        }
                        if is_key_down(KeyCode::Down) {
                            field.move_down();
                        }
                        self.last_move_time = now;
                    }
                }

                // 回転
                if is_key_pressed(KeyCode::X) {
                    field.rotate(Rotation::Right);
                }
                if is_key_pressed(KeyCode::Z) {
                    field.rotate(Rotation::Left);
                }

                // リセット
                if is_key_pressed(KeyCode::Escape) {
                    self.phase = GamePhase::Start;
                    return;
                }

                // 描画
                let cells = field.field();
                for row in 0..ROWS {
                    for col in 0..COLS {
                        if let Some(puyo) = cells[row][col] {
                            self.renderer.draw_puyo(puyo, col, row);
                        }
                    }
                }
                for (puyo, pos) in field.falling() {
                    self.renderer.draw_puyo(puyo, pos.col(), pos.row());
                }
            }
            GamePhase::GameOver(_field) => {
                self.renderer.draw_game_over();
                if is_key_pressed(KeyCode::Escape) {
                    self.phase = GamePhase::Start;
                }
            }
        }
    }
}
