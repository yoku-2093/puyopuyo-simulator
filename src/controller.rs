use crate::game::{GameField, PlayContext, Screen};
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

        field.tick(&mut self.ctx, now);

        if field.is_game_over() {
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
        for dp in field.draw_list(&self.ctx, get_time()) {
            if !dp.effect.visible {
                continue;
            }
            self.renderer.draw_puyo(
                dp.puyo,
                dp.col,
                dp.row,
                dp.effect.scale_x,
                dp.effect.scale_y,
            );
        }
        for p in field.particle_list() {
            let color = Color::new(p.color.r, p.color.g, p.color.b, p.alpha());
            self.renderer.draw_particle(p.col, p.row, p.size, color);
        }
    }
}
