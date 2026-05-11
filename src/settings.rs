/// ゲーム設定
pub struct Settings {
    pub puyo_colors: usize,    // 出現するぷよの色数（3〜5）
    pub bgm_volume: f32,       // BGM 音量（0.0〜1.0）
    pub se_volume: f32,        // 効果音音量（0.0〜1.0）
    pub showing_credits: bool, // クレジット表示中か
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            puyo_colors: 4,
            bgm_volume: 0.5,
            se_volume: 0.5,
            showing_credits: false,
        }
    }
}
