use crate::constants::*;
use crate::game::{GameField, PlayContext, PlayState, Screen};
use crate::render::Renderer;
use macroquad::prelude::*;

pub struct Controller {
    screen: Screen,
    renderer: Renderer,
    ctx: PlayContext,
}

impl Controller {
    pub async fn new() -> Self {
        let renderer = Renderer::new().await;
        Controller {
            screen: Screen::new(),
            renderer,
            ctx: PlayContext::new(),
        }
    }

    pub fn step(&mut self) {
        self.renderer.draw_background();
        self.renderer.draw_field();

        match &self.screen {
            Screen::Title => self.update_title(),
            Screen::Playing(_) => self.update_playing(),
            Screen::GameOver => self.update_game_over(),
        }
    }

    fn update_title(&mut self) {
        self.renderer.draw_title();
        self.renderer.draw_press_start();
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.screen = Screen::Playing(GameField::new());
            self.ctx = PlayContext::new();
        }
    }

    fn update_game_over(&mut self) {
        self.renderer.draw_game_over();
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Title;
        }
    }

    fn update_playing(&mut self) {
        let now = get_time();

        let Screen::Playing(field) = &mut self.screen else {
            return;
        };

        if field.tick(&mut self.ctx, now) {
            self.screen = Screen::GameOver;
            return;
        }

        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Title;
            return;
        }

        self.draw_playing();
    }

    fn draw_playing(&self) {
        let Screen::Playing(field) = &self.screen else {
            return;
        };
        let cells = field.field();
        for row in 0..ROWS {
            for col in 0..COLS {
                if let Some(puyo) = cells[row][col] {
                    self.renderer.draw_puyo(puyo, col as f32, row as f32);
                }
            }
        }
        if matches!(self.ctx.play_state, PlayState::Active | PlayState::Settling) {
            for (puyo, col, row) in field.active() {
                self.renderer.draw_puyo(puyo, col, row);
            }
        }
        for (puyo, col, row) in field.floating() {
            self.renderer.draw_puyo(puyo, col, row);
        }
    }
}
