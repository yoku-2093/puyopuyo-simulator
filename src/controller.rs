use crate::constants::*;
use crate::game::{GameField, PlayState, Screen};
use crate::puyo::Rotation;
use crate::render::Renderer;
use macroquad::prelude::*;

pub struct Controller {
    screen: Screen,
    renderer: Renderer,
    play_state: PlayState,
    last_move_time: f64,
    last_drop_time: f64,
    last_failed_rotation: Option<(Rotation, f64)>, // クイックターン判定用
}

impl Controller {
    pub async fn new() -> Self {
        let renderer = Renderer::new().await;
        Controller {
            screen: Screen::new(),
            renderer,
            play_state: PlayState::Active,
            last_drop_time: get_time(),
            last_move_time: 0.0,
            last_failed_rotation: None,
        }
    }

    pub fn step(&mut self) {
        self.renderer.draw_background();
        self.renderer.draw_field();

        match &mut self.screen {
            Screen::Title => {
                self.renderer.draw_title();
                self.renderer.draw_press_start();
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.screen = Screen::Playing(GameField::new());
                    self.last_drop_time = get_time();
                }
            }
            Screen::Playing(field) => {
                let now = get_time();

                match self.play_state {
                    PlayState::Active => {
                        // 自動落下
                        if now - self.last_drop_time > DROP_INTERVAL {
                            self.play_state = field.active_tick();
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
                            Controller::rotate(
                                field,
                                Rotation::Right,
                                &mut self.last_failed_rotation,
                            );
                        }
                        if is_key_pressed(KeyCode::Z) {
                            Controller::rotate(
                                field,
                                Rotation::Left,
                                &mut self.last_failed_rotation,
                            );
                        }
                    }
                    PlayState::Dropping => {
                        if now - self.last_drop_time > DROP_GRAVITY_INTERVAL {
                            if field.drop_tick() {
                                self.play_state = PlayState::Landed;
                            }
                            self.last_drop_time = now;
                        }
                    }
                    PlayState::Landed => {
                        if field.is_game_over() {
                            self.screen = Screen::GameOver(GameField::new());
                            return;
                        }
                        self.play_state = PlayState::Active;
                        field.spawn_next();
                        self.last_drop_time = now;
                    }
                }

                // リセット
                if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Title;
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
                if self.play_state == PlayState::Active {
                    for (puyo, pos) in field.active() {
                        self.renderer.draw_puyo(puyo, pos.col(), pos.row());
                    }
                }
            }
            Screen::GameOver(_field) => {
                self.renderer.draw_game_over();
                if is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Title;
                }
            }
        }
    }

    pub fn rotate(
        field: &mut GameField,
        rotation: Rotation,
        last_failed_rotation: &mut Option<(Rotation, f64)>,
    ) {
        let now = macroquad::time::get_time();
        let is_quick = matches!(
            *last_failed_rotation,
            Some((r, t)) if r == rotation && now - t < QUICK_TURN_WINDOW
        );

        if field.rotate(rotation, is_quick) {
            *last_failed_rotation = None;
        } else {
            *last_failed_rotation = Some((rotation, now));
        }
    }
}
