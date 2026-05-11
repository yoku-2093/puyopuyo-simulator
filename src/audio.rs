use macroquad::audio::{PlaySoundParams, Sound, load_sound, play_sound, set_sound_volume};

pub struct Audio {
    bgm: Sound,
    puyo_se: Sound,
}

impl Audio {
    pub async fn new() -> Self {
        let bgm = load_sound("assets/music/bgm.ogg").await.unwrap();
        let puyo_se = load_sound("assets/sounds/puyo.ogg").await.unwrap();
        Audio { bgm, puyo_se }
    }

    /// BGM をループ再生開始
    pub fn start_bgm(&self, volume: f32) {
        play_sound(
            &self.bgm,
            PlaySoundParams {
                looped: true,
                volume: volume,
            },
        );
    }

    /// BGM の音量を変更（再生中の音量を更新）
    pub fn set_bgm_volume(&self, volume: f32) {
        set_sound_volume(&self.bgm, volume);
    }

    /// ぷよ着地音を1回再生
    pub fn play_puyo(&self, volume: f32) {
        play_sound(
            &self.puyo_se,
            PlaySoundParams {
                looped: false,
                volume: volume,
            },
        );
    }
}
