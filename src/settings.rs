const KEY_PUYO_COLORS: &str = "puyo_colors";
const KEY_BGM_VOLUME: &str = "bgm_volume";
const KEY_SE_VOLUME: &str = "se_volume";

/// ゲーム設定
pub struct Settings {
    pub puyo_colors: usize,      // 出現するぷよの色数（3〜5）
    pub bgm_volume: f32,         // BGM 音量（0.0〜1.0）
    pub se_volume: f32,          // 効果音音量（0.0〜1.0）
    pub showing_credits: bool,   // クレジット表示中か（永続化しない）
    pub test_bgm_active: bool,   // 設定画面で BGM テスト中か（永続化しない）
    pub focused_index: usize,    // 設定画面の focus 位置（永続化しない）
}

impl Settings {
    /// 永続化から復元（無ければデフォルト）
    pub fn load() -> Self {
        let storage = quad_storage::STORAGE.lock().unwrap();
        let puyo_colors = storage
            .get(KEY_PUYO_COLORS)
            .and_then(|s| s.parse().ok())
            .unwrap_or(4);
        let bgm_volume = storage
            .get(KEY_BGM_VOLUME)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5);
        let se_volume = storage
            .get(KEY_SE_VOLUME)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.5);

        Settings {
            puyo_colors,
            bgm_volume,
            se_volume,
            showing_credits: false,
            test_bgm_active: false,
            focused_index: 0,
        }
    }

    /// 現在の設定を永続化
    pub fn save(&self) {
        let mut storage = quad_storage::STORAGE.lock().unwrap();
        storage.set(KEY_PUYO_COLORS, &self.puyo_colors.to_string());
        storage.set(KEY_BGM_VOLUME, &self.bgm_volume.to_string());
        storage.set(KEY_SE_VOLUME, &self.se_volume.to_string());
    }

    /// 設定画面の入力ハンドラ。
    /// 副作用が必要な操作 (テスト音再生、Close) は `SettingsEvent` で返す。
    pub fn handle_input(&mut self, input: SettingsInput) -> Option<SettingsEvent> {
        if self.showing_credits {
            if input.activate {
                self.showing_credits = false;
                self.focused_index = 0;
            }
            return None;
        }

        const WIDGET_COUNT: usize = 7;
        const VOLUME_STEP: f32 = 0.005;
        if input.navigate_prev {
            self.focused_index = (self.focused_index + WIDGET_COUNT - 1) % WIDGET_COUNT;
        }
        if input.navigate_next {
            self.focused_index = (self.focused_index + 1) % WIDGET_COUNT;
        }

        match self.focused_index {
            0 => {
                // Puyo colors (離散: 1 押しで 1 step)
                if input.right_pressed && self.puyo_colors < 5 {
                    self.puyo_colors += 1;
                }
                if input.left_pressed && self.puyo_colors > 3 {
                    self.puyo_colors -= 1;
                }
            }
            1 => {
                // BGM volume (連続: 押しっぱで増減)
                if input.right_held {
                    self.bgm_volume = (self.bgm_volume + VOLUME_STEP).min(1.0);
                }
                if input.left_held {
                    self.bgm_volume = (self.bgm_volume - VOLUME_STEP).max(0.0);
                }
            }
            2 => {
                // Test/Stop BGM toggle
                if input.activate {
                    self.test_bgm_active = !self.test_bgm_active;
                }
            }
            3 => {
                // SE volume (連続)
                if input.right_held {
                    self.se_volume = (self.se_volume + VOLUME_STEP).min(1.0);
                }
                if input.left_held {
                    self.se_volume = (self.se_volume - VOLUME_STEP).max(0.0);
                }
            }
            4 => {
                // Test SE button
                if input.activate {
                    return Some(SettingsEvent::TestSe);
                }
            }
            5 => {
                // Credits link
                if input.activate {
                    self.showing_credits = true;
                    self.focused_index = 0;
                }
            }
            6 => {
                // Back
                if input.activate {
                    return Some(SettingsEvent::Close);
                }
            }
            _ => {}
        }
        None
    }

    /// Settings 画面を出る時の状態リセット
    pub fn reset_ui_state(&mut self) {
        self.test_bgm_active = false;
        self.showing_credits = false;
        self.focused_index = 0;
    }
}

/// 設定画面の抽象化された入力。
/// Controller がキーボード状態を読んで埋める。
#[derive(Default, Clone, Copy)]
pub struct SettingsInput {
    pub navigate_prev: bool,
    pub navigate_next: bool,
    pub left_pressed: bool,
    pub left_held: bool,
    pub right_pressed: bool,
    pub right_held: bool,
    pub activate: bool,
}

/// Settings から外部 (Controller) で処理が必要なイベント
pub enum SettingsEvent {
    /// SE テスト音を再生してほしい
    TestSe,
    /// Settings を閉じたい
    Close,
}
