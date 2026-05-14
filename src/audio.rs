use macroquad::audio::{
    PlaySoundParams, Sound, load_sound, play_sound, set_sound_volume, stop_sound,
};

pub struct Audio {
    bgm: Sound,
    puyo_se: Sound,
    bgm_playing: bool,
}

impl Audio {
    pub async fn new() -> Self {
        let bgm = load_sound("assets/music/bgm.ogg").await.unwrap();
        let puyo_se = load_sound("assets/sounds/puyo.ogg").await.unwrap();
        Audio {
            bgm,
            puyo_se,
            bgm_playing: false,
        }
    }

    /// BGM の desired state を宣言的に設定する。
    /// 既に同じ状態なら音量だけ反映、状態が変わるなら start/stop する。
    pub fn set_bgm(&mut self, playing: bool, volume: f32) {
        match (self.bgm_playing, playing) {
            (false, true) => {
                play_sound(
                    &self.bgm,
                    PlaySoundParams {
                        looped: true,
                        volume,
                    },
                );
                self.bgm_playing = true;
            }
            (true, false) => {
                stop_sound(&self.bgm);
                self.bgm_playing = false;
            }
            (true, true) => {
                set_sound_volume(&self.bgm, volume);
            }
            (false, false) => {}
        }
    }

    /// ぷよ着地音を1回再生
    pub fn play_puyo(&self, volume: f32) {
        play_sound(
            &self.puyo_se,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }
}
