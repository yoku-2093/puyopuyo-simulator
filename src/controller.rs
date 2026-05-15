use crate::audio::Audio;
use crate::game::{COLS, GameEvent, GameField, PlayContext, ROWS};
use crate::render::{NextPuyo, Renderer};
use crate::settings::{Settings, SettingsEvent, SettingsInput};
use macroquad::prelude::*;

pub enum Screen {
    Title,              // タイトル画面
    Playing(GameField), // プレイ中
    GameOver,           // ゲームオーバー
    Settings,           // 設定画面
}

impl Screen {
    pub fn new() -> Self {
        Screen::Title
    }
}

pub struct Controller {
    screen: Screen,
    renderer: Renderer,
    ctx: PlayContext,
    settings: Settings,
    audio: Audio,
}

impl Controller {
    pub async fn new(window_width: f32, window_height: f32) -> Self {
        let renderer = Renderer::new(window_width, window_height, COLS, ROWS).await;
        let audio = Audio::new().await;
        let settings = Settings::load();
        Controller {
            screen: Screen::new(),
            renderer,
            ctx: PlayContext::new(),
            settings,
            audio,
        }
    }

    pub fn step(&mut self) {
        self.renderer.draw_background();
        self.renderer.draw_field();
        match &self.screen {
            Screen::Title => self.update_title(),
            Screen::Playing(_) => self.update_playing(),
            Screen::GameOver => self.update_game_over(),
            Screen::Settings => self.update_settings(),
        }
    }

    fn update_title(&mut self) {
        self.renderer.draw_press_start();
        self.audio.set_bgm(false, self.settings.bgm_volume);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.screen = Screen::Playing(GameField::new(self.settings.puyo_colors));
            self.ctx = PlayContext::new();
        } else if is_key_pressed(KeyCode::S) {
            self.settings.showing_credits = false;
            self.screen = Screen::Settings;
        }
    }

    fn update_settings(&mut self) {
        // キーボード状態を抽象入力にまとめる (widget の意味は知らない)
        let input = SettingsInput {
            navigate_prev: is_key_pressed(KeyCode::Up),
            navigate_next: is_key_pressed(KeyCode::Down),
            left_pressed: is_key_pressed(KeyCode::Left),
            left_held: is_key_down(KeyCode::Left),
            right_pressed: is_key_pressed(KeyCode::Right),
            right_held: is_key_down(KeyCode::Right),
            activate: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space),
        };

        // Settings に入力を渡し、副作用イベントを受け取る
        match self.settings.handle_input(input) {
            Some(SettingsEvent::TestSe) => {
                self.audio.play_puyo(self.settings.se_volume);
            }
            Some(SettingsEvent::Close) => {
                self.close_settings();
                return;
            }
            None => {}
        }

        // ESC はどの状態でも閉じる
        if is_key_pressed(KeyCode::Escape) {
            self.close_settings();
            return;
        }

        // 描画 + BGM 状態反映
        self.renderer.draw_settings(
            self.settings.puyo_colors,
            self.settings.bgm_volume,
            self.settings.se_volume,
            self.settings.showing_credits,
            self.settings.test_bgm_active,
            self.settings.focused_index,
        );
        self.audio
            .set_bgm(self.settings.test_bgm_active, self.settings.bgm_volume);
    }

    fn close_settings(&mut self) {
        self.settings.reset_ui_state();
        self.audio.set_bgm(false, self.settings.bgm_volume);
        self.settings.save();
        self.screen = Screen::Title;
    }

    fn update_game_over(&mut self) {
        self.renderer.draw_game_over();
        self.audio.set_bgm(false, self.settings.bgm_volume);
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Title;
        }
    }

    fn update_playing(&mut self) {
        let now = get_time();
        self.audio.set_bgm(true, self.settings.bgm_volume);

        let Screen::Playing(field) = &mut self.screen else {
            return;
        };

        field.tick(&mut self.ctx, now);
        field.update(&mut self.ctx, now);

        // ゲームから発生したイベントを処理
        for event in field.drain_events() {
            match event {
                GameEvent::PuyoLanded => self.audio.play_puyo(self.settings.se_volume),
                GameEvent::ChainPop { count, col, row } => {
                    self.audio.play_pop(self.settings.se_volume);
                    self.renderer.start_chain_effect(count, col, row);
                }
                GameEvent::GameOver => self.audio.play_game_over(self.settings.se_volume),
            }
        }

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

    fn draw_playing(&mut self) {
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

        let generation = field.spawn_count();
        let next = field.next();
        let nn = field.next_next();
        self.renderer.draw_next_puyos(
            &NextPuyo::new(next.axis(), next.child()),
            &NextPuyo::new(nn.axis(), nn.child()),
            generation,
        );
        self.renderer.draw_next_area();
        self.renderer.draw_score(field.score());
        self.renderer.draw_chain_effect();
    }
}
