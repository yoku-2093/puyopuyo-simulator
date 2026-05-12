use crate::audio::Audio;
use crate::game::{COLS, GameEvent, GameField, PlayContext, ROWS};
use crate::render::{NextPuyo, Renderer};
use crate::settings::Settings;
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
        audio.start_bgm(settings.bgm_volume);
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
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.screen = Screen::Playing(GameField::new(self.settings.puyo_colors));
            self.ctx = PlayContext::new();
        } else if is_key_pressed(KeyCode::S) {
            self.settings.showing_credits = false;
            self.screen = Screen::Settings;
        }
    }

    fn update_settings(&mut self) {
        let result = self.renderer.draw_settings(
            &mut self.settings.puyo_colors,
            &mut self.settings.bgm_volume,
            &mut self.settings.se_volume,
            &mut self.settings.showing_credits,
        );
        // BGM 音量を反映
        self.audio.set_bgm_volume(self.settings.bgm_volume);
        // SE 調整完了時にテスト音を鳴らす
        if result.test_se {
            self.audio.play_puyo(self.settings.se_volume);
        }
        if result.close || is_key_pressed(KeyCode::Escape) {
            self.settings.save();
            self.screen = Screen::Title;
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
        field.update(&mut self.ctx, now);

        // ゲームから発生したイベントを処理
        for event in field.drain_events() {
            match event {
                GameEvent::PuyoLanded => self.audio.play_puyo(self.settings.se_volume),
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
    }
}
