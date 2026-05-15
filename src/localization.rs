/// 表示言語。Settings に保持される。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Ja,
}

impl Lang {
    /// 永続化用の短い文字列識別子
    pub fn id(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ja => "ja",
        }
    }

    pub fn from_id(s: &str) -> Option<Self> {
        match s {
            "en" => Some(Lang::En),
            "ja" => Some(Lang::Ja),
            _ => None,
        }
    }

    /// 設定画面で表示する自言語名
    pub fn display_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Ja => "日本語",
        }
    }

}

/// 全表示文字列を保持する。新しい表示テキストを足すときはここにフィールド追加。
pub struct Strings {
    // タイトル画面
    pub title_press_start: &'static str,
    pub title_press_settings: &'static str,
    pub title_hint_move: &'static str,
    pub title_hint_softdrop: &'static str,
    pub title_hint_rotate_left: &'static str,
    pub title_hint_rotate_right: &'static str,

    // Pause / GameOver メニュー
    pub pause_title: &'static str,
    pub menu_resume: &'static str,
    pub menu_retry: &'static str,
    pub menu_retry_same: &'static str,
    pub menu_back_to_title: &'static str,
    pub menu_hint_pause: &'static str,
    pub menu_hint_gameover: &'static str,

    // Settings 画面
    pub settings_title: &'static str,
    pub settings_puyo_colors: &'static str,
    pub settings_bgm_volume: &'static str,
    pub settings_se_volume: &'static str,
    pub settings_language: &'static str,
    pub settings_credits: &'static str,
    pub settings_back: &'static str,

    // Settings 画面のヒントパーツ (組み合わせて 2 行のヒントを作る)
    pub hint_navigate: &'static str,    // "Navigate: ↑/↓"
    pub hint_adjust: &'static str,      // "Adjust: ←/→"
    pub hint_back: &'static str,        // "Back: Esc"
    pub hint_enter_kw: &'static str,    // "Enter / Space" (verb の後ろにつける)
    pub action_test_bgm: &'static str,  // "Test BGM"
    pub action_stop_bgm: &'static str,  // "Stop BGM"
    pub action_test_se: &'static str,   // "Test SE"
    pub action_choose_language: &'static str,
    pub action_show_credits: &'static str,
    pub action_close: &'static str,

    // Credits 画面
    pub credits_title: &'static str,
    pub credits_bgm_label: &'static str,
    pub credits_se_label: &'static str,
}

const EN: Strings = Strings {
    title_press_start: "PRESS ENTER or SPACE",
    title_press_settings: "Press S for Settings",
    title_hint_move: "Move: \u{2190} / \u{2192}",
    title_hint_softdrop: "Soft Drop: \u{2193}",
    title_hint_rotate_left: "Rotate Left: Z",
    title_hint_rotate_right: "Rotate Right: X",

    pause_title: "PAUSE",
    menu_resume: "Resume",
    menu_retry: "Retry",
    menu_retry_same: "Retry (same puyos)",
    menu_back_to_title: "Back to Title",
    menu_hint_pause: "\u{2191}/\u{2193} Select   Enter/Space Confirm   Esc Resume",
    menu_hint_gameover: "\u{2191}/\u{2193} Select   Enter/Space Confirm   Esc Title",

    settings_title: "Settings",
    settings_puyo_colors: "Puyo colors",
    settings_bgm_volume: "BGM volume",
    settings_se_volume: "SE volume",
    settings_language: "Language",
    settings_credits: "Credits",
    settings_back: "Back",

    hint_navigate: "Navigate: \u{2191} / \u{2193}",
    hint_adjust: "Adjust: \u{2190} / \u{2192}",
    hint_back: "Back: Esc",
    hint_enter_kw: "Enter / Space",
    action_test_bgm: "Test BGM",
    action_stop_bgm: "Stop BGM",
    action_test_se: "Test SE",
    action_choose_language: "Choose Language",
    action_show_credits: "Show Credits",
    action_close: "Close",

    credits_title: "Credits",
    credits_bgm_label: "BGM",
    credits_se_label: "SE",
};

const JA: Strings = Strings {
    title_press_start: "ENTER または SPACE で開始",
    title_press_settings: "S キーで設定",
    title_hint_move: "移動: \u{2190} / \u{2192}",
    title_hint_softdrop: "落下加速: \u{2193}",
    title_hint_rotate_left: "左回転: Z",
    title_hint_rotate_right: "右回転: X",

    pause_title: "ポーズ",
    menu_resume: "続ける",
    menu_retry: "リトライ",
    menu_retry_same: "同じぷよでリトライ",
    menu_back_to_title: "タイトルに戻る",
    menu_hint_pause: "\u{2191}/\u{2193} 選択   Enter/Space 決定   Esc 再開",
    menu_hint_gameover: "\u{2191}/\u{2193} 選択   Enter/Space 決定   Esc タイトル",

    settings_title: "設定",
    settings_puyo_colors: "ぷよの色数",
    settings_bgm_volume: "BGM 音量",
    settings_se_volume: "効果音 音量",
    settings_language: "言語",
    settings_credits: "クレジット",
    settings_back: "戻る",

    hint_navigate: "移動: \u{2191} / \u{2193}",
    hint_adjust: "調整: \u{2190} / \u{2192}",
    hint_back: "戻る: Esc",
    hint_enter_kw: "Enter / Space",
    action_test_bgm: "BGM 再生",
    action_stop_bgm: "BGM 停止",
    action_test_se: "効果音 再生",
    action_choose_language: "言語を選ぶ",
    action_show_credits: "クレジットを開く",
    action_close: "閉じる",

    credits_title: "クレジット",
    credits_bgm_label: "BGM",
    credits_se_label: "SE",
};

pub fn strings(lang: Lang) -> &'static Strings {
    match lang {
        Lang::En => &EN,
        Lang::Ja => &JA,
    }
}

/// 言語に応じて連鎖表示の文字列を組み立てる。
pub fn format_chain(count: u32, lang: Lang) -> String {
    match lang {
        Lang::En => format!("{count} Chain!"),
        Lang::Ja => format!("{count}連鎖"),
    }
}
