const KEY_PUYO_COLORS: &str = "puyo_colors";
const KEY_BGM_VOLUME: &str = "bgm_volume";
const KEY_SE_VOLUME: &str = "se_volume";

/// ゲーム設定
pub struct Settings {
    pub puyo_colors: usize,    // 出現するぷよの色数（3〜5）
    pub bgm_volume: f32,       // BGM 音量（0.0〜1.0）
    pub se_volume: f32,        // 効果音音量（0.0〜1.0）
    pub showing_credits: bool, // クレジット表示中か（永続化しない）
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
        }
    }

    /// 現在の設定を永続化
    pub fn save(&self) {
        let mut storage = quad_storage::STORAGE.lock().unwrap();
        storage.set(KEY_PUYO_COLORS, &self.puyo_colors.to_string());
        storage.set(KEY_BGM_VOLUME, &self.bgm_volume.to_string());
        storage.set(KEY_SE_VOLUME, &self.se_volume.to_string());
    }
}
