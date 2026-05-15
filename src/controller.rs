use crate::audio::Audio;
use crate::game::{COLS, GameEvent, GameField, PlayContext, ROWS};
use crate::render::{NextPuyo, Renderer};
use crate::settings::{Settings, SettingsEvent, SettingsInput};
use macroquad::prelude::*;

pub enum Screen {
    Title,
    Playing(GameField),
    Paused {
        field: GameField,
        focused_index: usize,
    },
    GameOver {
        focused_index: usize,
    },
    Settings,
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
    /// 直近の seed (「同じぷよでリトライ」用)
    last_seed: Option<u64>,
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
            last_seed: None,
        }
    }

    pub fn step(&mut self) {
        // Settings 画面で言語を切り替えた瞬間から反映されるよう、毎フレーム同期
        self.renderer.set_lang(self.settings.lang);
        self.renderer.draw_background();
        self.renderer.draw_field();
        match &self.screen {
            Screen::Title => self.update_title(),
            Screen::Playing(_) => self.update_playing(),
            Screen::Paused { .. } => self.update_paused(),
            Screen::GameOver { .. } => self.update_game_over(),
            Screen::Settings => self.update_settings(),
        }
    }

    fn update_title(&mut self) {
        self.renderer.draw_press_start();
        self.audio.set_bgm(false, self.settings.bgm_volume);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.start_game(None);
        } else if is_key_pressed(KeyCode::S) {
            self.settings.showing_credits = false;
            self.screen = Screen::Settings;
        }
    }

    /// 新しいゲームを開始する。`seed = None` なら新規 seed 生成、`Some` なら指定 seed で再現プレイ。
    fn start_game(&mut self, seed: Option<u64>) {
        let seed = seed.unwrap_or_else(|| {
            // u32 を 2 回叩いて u64 を作る
            ((rand::rand() as u64) << 32) | (rand::rand() as u64)
        });
        self.last_seed = Some(seed);
        self.screen = Screen::Playing(GameField::new(self.settings.puyo_colors, seed));
        self.ctx = PlayContext::new();
    }

    fn update_settings(&mut self) {
        let input = SettingsInput {
            navigate_prev: is_key_pressed(KeyCode::Up),
            navigate_next: is_key_pressed(KeyCode::Down),
            left_pressed: is_key_pressed(KeyCode::Left),
            left_held: is_key_down(KeyCode::Left),
            right_pressed: is_key_pressed(KeyCode::Right),
            right_held: is_key_down(KeyCode::Right),
            activate: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space),
        };

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

        // ESC はどの状態でも Settings 自体を閉じる
        if is_key_pressed(KeyCode::Escape) {
            self.close_settings();
            return;
        }

        self.renderer.draw_settings(
            self.settings.puyo_colors,
            self.settings.bgm_volume,
            self.settings.se_volume,
            self.settings.showing_credits,
            self.settings.test_bgm_active,
            self.settings.focused_index,
            self.settings.showing_language_picker,
            self.settings.lang_picker_index,
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
        const ITEM_COUNT: usize = 3;
        if let Screen::GameOver { focused_index } = &mut self.screen {
            if is_key_pressed(KeyCode::Up) {
                *focused_index = (*focused_index + ITEM_COUNT - 1) % ITEM_COUNT;
            }
            if is_key_pressed(KeyCode::Down) {
                *focused_index = (*focused_index + 1) % ITEM_COUNT;
            }
        }
        let focused_index = match &self.screen {
            Screen::GameOver { focused_index } => *focused_index,
            _ => return,
        };

        self.audio.set_bgm(false, self.settings.bgm_volume);
        self.renderer.draw_game_over(focused_index);

        // Esc は仕様としてタイトル直行（既存挙動を維持）
        if is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Title;
            return;
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            match focused_index {
                0 => self.start_game(None),
                1 => self.start_game(self.last_seed),
                2 => self.screen = Screen::Title,
                _ => {}
            }
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
            self.screen = Screen::GameOver { focused_index: 0 };
            return;
        }

        if is_key_pressed(KeyCode::Escape) {
            self.pause_game();
            return;
        }

        self.draw_playing();
    }

    /// Playing から Pause に遷移（GameField を保持したまま）
    fn pause_game(&mut self) {
        let prev = std::mem::replace(&mut self.screen, Screen::Title);
        if let Screen::Playing(field) = prev {
            self.screen = Screen::Paused {
                field,
                focused_index: 0,
            };
        }
    }

    /// Pause から Playing に戻す
    fn resume_game(&mut self) {
        let prev = std::mem::replace(&mut self.screen, Screen::Title);
        if let Screen::Paused { field, .. } = prev {
            self.screen = Screen::Playing(field);
        }
    }

    fn update_paused(&mut self) {
        const ITEM_COUNT: usize = 4;
        // フォーカス更新
        if let Screen::Paused { focused_index, .. } = &mut self.screen {
            if is_key_pressed(KeyCode::Up) {
                *focused_index = (*focused_index + ITEM_COUNT - 1) % ITEM_COUNT;
            }
            if is_key_pressed(KeyCode::Down) {
                *focused_index = (*focused_index + 1) % ITEM_COUNT;
            }
        }
        let focused_index = match &self.screen {
            Screen::Paused { focused_index, .. } => *focused_index,
            _ => return,
        };

        // BGM は止めておく（プレイ中の没入を切るため）
        self.audio.set_bgm(false, self.settings.bgm_volume);

        // 凍結したゲーム状態を背景として描画
        let now = get_time();
        if let Screen::Paused { field, .. } = &self.screen {
            Self::draw_field_state(&mut self.renderer, field, &self.ctx, now);
        }
        self.renderer.draw_pause_menu(focused_index);

        // 入力ディスパッチ
        let activate = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if is_key_pressed(KeyCode::Escape) {
            self.resume_game();
            return;
        }
        if activate {
            match focused_index {
                0 => self.resume_game(),
                1 => self.start_game(None),
                2 => self.start_game(self.last_seed),
                3 => self.screen = Screen::Title,
                _ => {}
            }
        }
    }

    fn draw_playing(&mut self) {
        let Screen::Playing(field) = &self.screen else {
            return;
        };
        Self::draw_field_state(&mut self.renderer, field, &self.ctx, get_time());
    }

    /// Playing / Paused 共通のフィールド描画。レンダラとフィールドを直接受け取り、
    /// 呼び出し側の `&self.screen` 借用と衝突しないようにする。
    fn draw_field_state(renderer: &mut Renderer, field: &GameField, ctx: &PlayContext, now: f64) {
        for dp in field.draw_list(ctx, now) {
            if !dp.effect.visible {
                continue;
            }
            renderer.draw_puyo(
                dp.puyo,
                dp.col,
                dp.row,
                dp.effect.scale_x,
                dp.effect.scale_y,
            );
        }
        for p in field.particle_list() {
            let color = Color::new(p.color.r, p.color.g, p.color.b, p.alpha());
            renderer.draw_particle(p.col, p.row, p.size, color);
        }

        let generation = field.spawn_count();
        let next = field.next();
        let nn = field.next_next();
        renderer.draw_next_puyos(
            &NextPuyo::new(next.axis(), next.child()),
            &NextPuyo::new(nn.axis(), nn.child()),
            generation,
        );
        renderer.draw_next_area();
        renderer.draw_score(field.score());
        renderer.draw_chain_effect();
    }
}
