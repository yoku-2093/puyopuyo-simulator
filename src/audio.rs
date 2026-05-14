use macroquad::audio::{
    PlaySoundParams, Sound, load_sound, play_sound, set_sound_volume, stop_sound,
};
use macroquad::experimental::coroutines::{start_coroutine, wait_seconds};

const POP_DELAY: f32 = 0.1; // 連鎖確定からポップ音再生までの遅延（秒）

pub struct Audio {
    bgm: Sound,
    puyo_se: Sound,
    pop_se: Sound,
    game_over_se: Sound,
    bgm_playing: bool,
}

impl Audio {
    pub async fn new() -> Self {
        let bgm = load_sound("assets/audio/bgm.ogg").await.unwrap();
        let puyo_se = load_sound("assets/audio/puyo.ogg").await.unwrap();
        let pop_se = load_sound("assets/audio/pop.ogg").await.unwrap();
        let game_over_se = load_sound("assets/audio/game_over.ogg").await.unwrap();
        Audio {
            bgm,
            puyo_se,
            pop_se,
            game_over_se,
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

    pub fn play_puyo(&self, volume: f32) {
        play_one_shot(&self.puyo_se, volume);
    }

    pub fn play_pop(&self, volume: f32) {
        let sound = self.pop_se.clone();
        start_coroutine(async move {
            wait_seconds(POP_DELAY).await;
            play_one_shot(&sound, volume);
        });
    }

    pub fn play_game_over(&self, volume: f32) {
        play_one_shot(&self.game_over_se, volume);
    }
}

fn play_one_shot(sound: &Sound, volume: f32) {
    play_sound(
        sound,
        PlaySoundParams {
            looped: false,
            volume,
        },
    );
}
