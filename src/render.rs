use crate::localization::{self, Lang, Strings};
use crate::types::Puyo;
use macroquad::prelude::*;
use std::collections::HashMap;

const PUYO_SIZE: f32 = 60.0; // ぷよ1個あたりの描画サイズ（ピクセル）
const FIELD_PADDING: f32 = 20.0; // フィールド外枠の余白（ピクセル）
const NEXT_ANIM_DURATION: f64 = 0.15; // ネクスト遷移アニメーションの長さ（秒）

const JAPANESE_FONT: &[u8] = include_bytes!("../assets/fonts/NotoSansJP.ttf");

// ===== Settings / Credits 画面のレイアウト定数 =====
// 「DY」「DX」は panel の左上 (panel_x, panel_y) からのオフセット

// パネル本体
const PANEL_W: f32 = 480.0;
const PANEL_H: f32 = 480.0;
const PANEL_BORDER: f32 = 2.0;
const PANEL_BG: Color = Color::new(0.0, 0.0, 0.0, 0.85);

// タイトル ("Settings" / "Credits")
const PANEL_TITLE_DY: f32 = 40.0;
const PANEL_TITLE_FONT: u16 = 22;

// ナビゲーションヒント (Settings 画面のみ)
const HINT_LINE1_DY: f32 = 80.0;
const HINT_LINE2_DY: f32 = 97.0;
const HINT_FONT: u16 = 12;

// スライダー (Settings 画面)
const SLIDER_TOP_DY: f32 = 130.0;
const SLIDER_ROW_H: f32 = 48.0;
const SLIDER_LABEL_DX: f32 = 50.0;
const SLIDER_BAR_DX: f32 = 165.0;
const SLIDER_BAR_W: f32 = 175.0;
const SLIDER_VALUE_DX: f32 = 355.0;
const FOCUS_MARKER_GAP: f32 = 22.0; // ラベルの左に置く focus marker のオフセット
const VALUE_FONT: u16 = 14;

// Back ボタン (Settings 画面下部)
const BACK_BTN_W: f32 = 140.0;
const BACK_BTN_H: f32 = 32.0;
const BACK_BTN_BOTTOM_GAP: f32 = 60.0;

// Credits ボタン (Credits 画面の "Back" は別サイズ)
const CRED_BACK_W: f32 = 120.0;
const CRED_BACK_H: f32 = 32.0;

// Credits 画面の項目
const CRED_CAT_DX: f32 = 60.0;
const CRED_VAL_DX: f32 = 140.0;
const CRED_BGM_ROW_DY: f32 = 110.0;
const CRED_SE_ROW_DY: f32 = 150.0;
const CRED_CAT_FONT: u16 = 13;
const CRED_VAL_FONT: u16 = 14;
const CRED_CAT_COLOR: Color = Color::new(0.6, 0.6, 0.6, 1.0);

// ヒント文字色
const HINT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.65);

// ===== テキスト共通 =====
// 縁取り 8 方向の単位オフセット（実際の太さは draw_outlined_text の引数で乗算）
const OUTLINE_OFFSETS: [(f32, f32); 8] = [
    (-1.0, 0.0),
    (1.0, 0.0),
    (0.0, -1.0),
    (0.0, 1.0),
    (-1.0, -1.0),
    (1.0, -1.0),
    (-1.0, 1.0),
    (1.0, 1.0),
];
const TEXT_HIGHLIGHT_COLOR: Color = Color::new(1.0, 0.95, 0.2, 1.0); // 強調黄色（score, chain）
const TEXT_OUTLINE_COLOR: Color = Color::new(0.0, 0.0, 0.0, 1.0);

// ===== Score 表示 =====
const SCORE_FONT: u16 = 36;
const SCORE_OUTLINE: f32 = 2.0;

// ===== Pause / GameOver 共通メニュー =====
const MENU_OVERLAY_ALPHA: f32 = 0.6;
const MENU_TITLE_DY: f32 = -160.0; // フィールド中央からのオフセット
const MENU_TITLE_FONT: u16 = 48;
const MENU_FIRST_ITEM_DY: f32 = -40.0;
const MENU_ITEM_FONT: u16 = 22;
const MENU_ITEM_GAP: f32 = 40.0;
const MENU_HINT_DY: f32 = 200.0;
const MENU_HINT_FONT: u16 = 13;
const MENU_FOCUS_MARKER_GAP: f32 = 22.0; // ラベル左端から marker 中心までの距離
const MENU_NORMAL_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.9);

// ===== 連鎖数エフェクト =====
const CHAIN_DURATION: f64 = 1.2; // 表示の総時間（秒）
const CHAIN_FONT: u16 = 28; // ベースフォントサイズ（scale が掛かる）
const CHAIN_OUTLINE: f32 = 2.0;
const CHAIN_SCALE_PEAK: f32 = 1.2; // ポップ時のオーバーシュート倍率
const CHAIN_POP_END: f32 = 0.15; // progress: 0 → SCALE_PEAK
const CHAIN_SETTLE_END: f32 = 0.25; // progress: SCALE_PEAK → 1.0
const CHAIN_FADE_START: f32 = 0.7; // progress: ここから alpha フェード開始
const CHAIN_FLOAT_DR: f32 = 0.6; // 全体を通して上にフロートする量（セル単位）

pub struct NextPuyo {
    pub axis: Puyo,
    pub child: Puyo,
}

impl NextPuyo {
    pub fn new(axis: Puyo, child: Puyo) -> Self {
        NextPuyo { axis, child }
    }
}

struct NextAnim {
    start_time: f64,               // アニメーション開始時刻
    generation: u32,               // 世代（変化検出用）
    current: Option<(Puyo, Puyo)>, // 現在のネクスト
    exiting: Option<(Puyo, Puyo)>, // 上に出ていく旧ネクスト
}

impl NextAnim {
    fn new() -> Self {
        NextAnim {
            start_time: 0.0,
            generation: 0,
            current: None,
            exiting: None,
        }
    }
}

struct ChainEffect {
    count: u32,
    col: f32,
    row: f32,
    start_time: f64,
}

pub struct Renderer {
    textures: HashMap<Puyo, Texture2D>,
    background: Texture2D,
    field_bg: Texture2D,
    field: Texture2D,
    font: Font,
    game_over_text: Texture2D,
    next_area: Texture2D,
    window_width: f32,
    window_height: f32,
    cols: usize,
    rows: usize,
    field_x: f32,
    field_y: f32,
    next_anim: NextAnim,
    chain_effect: Option<ChainEffect>,
    lang: Lang,
}

impl Renderer {
    pub async fn new(window_width: f32, window_height: f32, cols: usize, rows: usize) -> Self {
        let puyos = [
            (Puyo::Blue, "assets/images/puyo/blue.png"),
            (Puyo::Green, "assets/images/puyo/green.png"),
            (Puyo::Red, "assets/images/puyo/red.png"),
            (Puyo::Yellow, "assets/images/puyo/yellow.png"),
            (Puyo::Purple, "assets/images/puyo/purple.png"),
        ];

        let mut textures = HashMap::new();
        for (puyo, path) in puyos {
            let texture = load_texture(path).await.unwrap();
            texture.set_filter(FilterMode::Nearest);
            textures.insert(puyo, texture);
        }

        let background = load_texture("assets/images/background/window.png")
            .await
            .unwrap();
        let field_bg = load_texture("assets/images/background/field_bg.png")
            .await
            .unwrap();
        let field = load_texture("assets/images/background/field.png")
            .await
            .unwrap();

        let font = load_ttf_font_from_bytes(JAPANESE_FONT).unwrap();

        let game_over_text = load_texture("assets/images/game_over.png").await.unwrap();
        game_over_text.set_filter(FilterMode::Linear);

        let next_area = load_texture("assets/images/background/next_area.png")
            .await
            .unwrap();
        next_area.set_filter(FilterMode::Nearest);

        let field_x = (window_width - PUYO_SIZE * cols as f32) / 2.0;
        let field_y = (window_height - PUYO_SIZE * rows as f32) / 2.0;

        Renderer {
            textures,
            background,
            field_bg,
            field,
            font,
            game_over_text,
            next_area,
            window_width,
            window_height,
            cols,
            rows,
            field_x,
            field_y,
            next_anim: NextAnim::new(),
            chain_effect: None,
            lang: Lang::En,
        }
    }

    /// 表示言語を切り替える（毎フレーム呼ばれて OK、変更がなければ実質ノーオペ）。
    pub fn set_lang(&mut self, lang: Lang) {
        self.lang = lang;
    }

    /// 現在のウィンドウサイズに合わせてレイアウトを更新する。
    /// 毎フレーム呼ばれる前提。フィールドを画面中央に配置する。
    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
        self.field_x = (width - PUYO_SIZE * self.cols as f32) / 2.0;
        self.field_y = (height - PUYO_SIZE * self.rows as f32) / 2.0;
    }

    fn s(&self) -> &'static Strings {
        localization::strings(self.lang)
    }

    pub fn draw_background(&self) {
        draw_texture_ex(
            &self.background,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.window_width, self.window_height)),
                ..Default::default()
            },
        );
    }

    pub fn draw_field(&self) {
        let field_w = PUYO_SIZE * self.cols as f32;
        let field_h = PUYO_SIZE * self.rows as f32;
        let padding = FIELD_PADDING;
        let bg_w = field_w + padding * 2.0;
        let bg_h = field_h + padding * 4.0;

        // 外枠（field_bg）をフィールドより一回り大きく描画
        draw_texture_ex(
            &self.field_bg,
            self.field_x - padding,
            self.field_y - padding,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(bg_w, bg_h)),
                ..Default::default()
            },
        );

        // フィールド本体（field）を中央に描画
        draw_texture_ex(
            &self.field,
            self.field_x,
            self.field_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(field_w, field_h)),
                ..Default::default()
            },
        );
    }

    /// フィールド中央にテキストを描画するヘルパー
    fn draw_centered_text(&self, text: &str, font_size: f32, color: Color, y_offset: f32) {
        let field_w = PUYO_SIZE * self.cols as f32;
        let field_h = PUYO_SIZE * self.rows as f32;
        let center_x = self.field_x + field_w / 2.0;
        let center_y = self.field_y + field_h / 2.0 + y_offset;

        let params = TextParams {
            font: Some(&self.font),
            font_size: font_size as u16,
            color,
            ..Default::default()
        };
        let dimensions = measure_text(text, Some(&self.font), font_size as u16, 1.0);
        draw_text_ex(
            text,
            (center_x - dimensions.width / 2.0).round(),
            (center_y + dimensions.height / 2.0).round(),
            params,
        );
    }

    /// スタート画面の描画
    pub fn draw_press_start(&self) {
        let s = self.s();
        let t = ((get_time() * 0.7 % 1.0) as f32 * 2.0 - 1.0).abs();
        let alpha = 0.4 + 0.6 * t * t * (3.0 - 2.0 * t);
        self.draw_centered_text(
            s.title_press_start,
            24.0,
            Color::new(1.0, 1.0, 0.0, alpha),
            -40.0,
        );
        self.draw_centered_text(
            s.title_press_settings,
            16.0,
            Color::new(1.0, 1.0, 1.0, 0.7),
            0.0,
        );
        // 操作方法
        self.draw_centered_text(s.title_hint_move, 13.0, HINT_COLOR, 60.0);
        self.draw_centered_text(s.title_hint_softdrop, 13.0, HINT_COLOR, 80.0);
        self.draw_centered_text(s.title_hint_rotate_left, 13.0, HINT_COLOR, 100.0);
        self.draw_centered_text(s.title_hint_rotate_right, 13.0, HINT_COLOR, 120.0);
    }

    /// Pause / GameOver 共通: フィールドに半透明の暗幕を被せる
    fn draw_menu_overlay(&self) {
        let field_w = PUYO_SIZE * self.cols as f32;
        let field_h = PUYO_SIZE * self.rows as f32;
        draw_rectangle(
            self.field_x,
            self.field_y,
            field_w,
            field_h,
            Color::new(0.0, 0.0, 0.0, MENU_OVERLAY_ALPHA),
        );
    }

    /// Pause / GameOver 共通: メニュー項目を縦に描画。フォーカス中の左に ">" を出す。
    fn draw_menu_items(&self, items: &[&str], focused_index: usize) {
        let field_w = PUYO_SIZE * self.cols as f32;
        let field_h = PUYO_SIZE * self.rows as f32;
        let center_x = self.field_x + field_w / 2.0;
        let center_y = self.field_y + field_h / 2.0;

        for (i, label) in items.iter().enumerate() {
            let dy = MENU_FIRST_ITEM_DY + (i as f32) * MENU_ITEM_GAP;
            let focused = i == focused_index;
            let color = if focused {
                TEXT_HIGHLIGHT_COLOR
            } else {
                MENU_NORMAL_COLOR
            };
            let dims = measure_text(label, Some(&self.font), MENU_ITEM_FONT, 1.0);
            let x = (center_x - dims.width / 2.0).round();
            let y = (center_y + dy + dims.height / 2.0).round();

            draw_text_ex(
                label,
                x,
                y,
                TextParams {
                    font: Some(&self.font),
                    font_size: MENU_ITEM_FONT,
                    color,
                    ..Default::default()
                },
            );
            if focused {
                draw_text_ex(
                    ">",
                    x - MENU_FOCUS_MARKER_GAP,
                    y,
                    TextParams {
                        font: Some(&self.font),
                        font_size: MENU_ITEM_FONT,
                        color: TEXT_HIGHLIGHT_COLOR,
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn draw_menu_hint(&self, text: &str) {
        self.draw_centered_text(
            text,
            MENU_HINT_FONT as f32,
            Color::new(1.0, 1.0, 1.0, 0.6),
            MENU_HINT_DY,
        );
    }

    /// ポーズ画面（プレイ中の Esc）
    pub fn draw_pause_menu(&self, focused_index: usize) {
        let s = self.s();
        self.draw_menu_overlay();
        self.draw_centered_text(
            s.pause_title,
            MENU_TITLE_FONT as f32,
            TEXT_HIGHLIGHT_COLOR,
            MENU_TITLE_DY,
        );
        self.draw_menu_items(
            &[
                s.menu_resume,
                s.menu_retry,
                s.menu_retry_same,
                s.menu_back_to_title,
            ],
            focused_index,
        );
        self.draw_menu_hint(s.menu_hint_pause);
    }

    /// ゲームオーバー画面
    pub fn draw_game_over(&self, focused_index: usize) {
        self.draw_menu_overlay();

        // GAME OVER 画像をタイトル位置にスケールアニメで描画
        let t = ((get_time() * 1.5).sin() * 0.5 + 0.5) as f32;
        let scale = 0.30 + 0.05 * t; // メニュー追加に伴い縮小
        let tex_w = self.game_over_text.width() * scale;
        let tex_h = self.game_over_text.height() * scale;
        let field_w = PUYO_SIZE * self.cols as f32;
        let field_h = PUYO_SIZE * self.rows as f32;
        let cx = self.field_x + field_w / 2.0 - tex_w / 2.0;
        let cy = self.field_y + field_h / 2.0 + MENU_TITLE_DY - tex_h / 2.0;
        draw_texture_ex(
            &self.game_over_text,
            cx,
            cy,
            Color::new(0.0, 0.2, 1.0, 1.0),
            DrawTextureParams {
                dest_size: Some(Vec2::new(tex_w, tex_h)),
                ..Default::default()
            },
        );

        let s = self.s();
        self.draw_menu_items(
            &[s.menu_retry, s.menu_retry_same, s.menu_back_to_title],
            focused_index,
        );
        self.draw_menu_hint(s.menu_hint_gameover);
    }

    pub fn draw_puyo(&self, puyo: Puyo, col: f32, row: f32, scale_x: f32, scale_y: f32) {
        let w = PUYO_SIZE * scale_x;
        let h = PUYO_SIZE * scale_y;
        let x = self.field_x + col * PUYO_SIZE - (w - PUYO_SIZE) / 2.0;
        let y = self.field_y + row * PUYO_SIZE + (PUYO_SIZE - h);
        draw_texture_ex(
            &self.textures[&puyo],
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(w, h)),
                ..Default::default()
            },
        );
    }

    /// ネクストエリアの位置とサイズ
    fn next_area_rect(&self) -> (f32, f32, f32, f32) {
        let field_w = PUYO_SIZE * self.cols as f32;
        let gap = 5.0;
        let area_x = self.field_x + field_w + FIELD_PADDING + gap;
        let area_y = self.field_y + FIELD_PADDING;
        let area_w = PUYO_SIZE * 2.5;
        let area_h = PUYO_SIZE * 5.5;
        (area_x, area_y, area_w, area_h)
    }

    /// ネクストエリアの背景を描画（常時表示）
    pub fn draw_next_area(&self) {
        let (area_x, area_y, area_w, area_h) = self.next_area_rect();
        draw_texture_ex(
            &self.next_area,
            area_x,
            area_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(area_w, area_h)),
                ..Default::default()
            },
        );
    }

    /// ネクスト・ネクネクのぷよをアニメーション付きで描画
    pub fn draw_next_puyos(&mut self, next: &NextPuyo, next_next: &NextPuyo, generation: u32) {
        let now = get_time();

        // 世代が変わったらアニメーション開始
        if self.next_anim.generation != generation {
            self.next_anim.exiting = self.next_anim.current;
            self.next_anim.start_time = now;
            self.next_anim.generation = generation;
        }
        self.next_anim.current = Some((next.axis, next.child));

        let (area_x, area_y, area_w, area_h) = self.next_area_rect();
        let textures = &self.textures;

        // ease-out 進行度
        let raw_t = ((now - self.next_anim.start_time) / NEXT_ANIM_DURATION).clamp(0.0, 1.0) as f32;
        let t = 1.0 - (1.0 - raw_t) * (1.0 - raw_t);
        let lerp = |a: f32, b: f32| a + (b - a) * t;

        // (位置, スケール) の定義
        let nn_scale = 0.75;
        let nn_size = PUYO_SIZE * nn_scale;
        let next_rest = (area_x + PUYO_SIZE * 0.3, area_y + PUYO_SIZE);
        let nn_rest = (
            area_x + area_w - nn_size - PUYO_SIZE * 0.3,
            area_y + PUYO_SIZE * 3.2,
        );
        let entry = (nn_rest.0, area_y + area_h + nn_size * 2.0);
        let exit = (next_rest.0, area_y - PUYO_SIZE * 2.5);

        // ネクストエリア矩形でクリップしてぷよを描画（はみ出しは source rect で切り取る）
        let draw_clipped = |puyo: Puyo, x: f32, y: f32, scale: f32| {
            let size = PUYO_SIZE * scale;
            let clip_right = area_x + area_w;
            let clip_bottom = area_y + area_h;
            if x + size <= area_x || x >= clip_right || y + size <= area_y || y >= clip_bottom {
                return;
            }
            let (vl, vr, vt, vb) = (
                (area_x - x).max(0.0),
                (clip_right - x).min(size),
                (area_y - y).max(0.0),
                (clip_bottom - y).min(size),
            );
            let (tex, tex_w, tex_h) = (
                &textures[&puyo],
                textures[&puyo].width(),
                textures[&puyo].height(),
            );
            draw_texture_ex(
                tex,
                x + vl,
                y + vt,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(vr - vl, vb - vt)),
                    source: Some(Rect::new(
                        vl / size * tex_w,
                        vt / size * tex_h,
                        (vr - vl) / size * tex_w,
                        (vb - vt) / size * tex_h,
                    )),
                    ..Default::default()
                },
            );
        };

        // (axis, child) のぷよ組を縦に並べて描画
        let draw_pair = |pair: (Puyo, Puyo), (x, y): (f32, f32), scale: f32| {
            let size = PUYO_SIZE * scale;
            draw_clipped(pair.1, x, y, scale);
            draw_clipped(pair.0, x, y + size, scale);
        };

        // ① 退出中の旧ネクスト: 安定位置 → 上へ
        if let Some(exiting) = self.next_anim.exiting {
            draw_pair(
                exiting,
                (lerp(next_rest.0, exit.0), lerp(next_rest.1, exit.1)),
                1.0,
            );
        }
        // ② ネクスト: ネクネク位置 → ネクスト位置（拡大しながら）
        let next_scale = nn_scale + (1.0 - nn_scale) * t;
        draw_pair(
            (next.axis, next.child),
            (lerp(nn_rest.0, next_rest.0), lerp(nn_rest.1, next_rest.1)),
            next_scale,
        );
        // ③ ネクネク: 入口 → ネクネク位置
        draw_pair(
            (next_next.axis, next_next.child),
            (lerp(entry.0, nn_rest.0), lerp(entry.1, nn_rest.1)),
            nn_scale,
        );
    }

    pub fn draw_particle(&self, col: f32, row: f32, size: f32, color: Color) {
        let x = self.field_x + col * PUYO_SIZE + PUYO_SIZE / 2.0;
        let y = self.field_y + row * PUYO_SIZE + PUYO_SIZE / 2.0;
        let r = size * PUYO_SIZE;
        draw_circle(x, y, r, color);
    }

    /// 縁取り付きでテキストを描画。outline は縁の太さ（ピクセル）。
    fn draw_outlined_text(
        &self,
        text: &str,
        x: f32,
        y: f32,
        font_size: u16,
        color: Color,
        outline_color: Color,
        outline: f32,
    ) {
        for (dx, dy) in OUTLINE_OFFSETS {
            draw_text_ex(
                text,
                x + dx * outline,
                y + dy * outline,
                TextParams {
                    font: Some(&self.font),
                    font_size,
                    color: outline_color,
                    ..Default::default()
                },
            );
        }
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(&self.font),
                font_size,
                color,
                ..Default::default()
            },
        );
    }

    /// 連鎖エフェクトを開始（消えたぷよの重心 col/row は visible grid 座標）。
    /// 既存のエフェクトがあれば上書きする。
    pub fn start_chain_effect(&mut self, count: u32, col: f32, row: f32) {
        self.chain_effect = Some(ChainEffect {
            count,
            col,
            row,
            start_time: get_time(),
        });
    }

    /// 連鎖エフェクトを描画（active かつ寿命内のときのみ）。
    pub fn draw_chain_effect(&mut self) {
        let Some(eff) = &self.chain_effect else {
            return;
        };
        let progress = ((get_time() - eff.start_time) / CHAIN_DURATION) as f32;
        if progress >= 1.0 {
            self.chain_effect = None;
            return;
        }

        // scale: 0..POP_END で 0→PEAK、POP_END..SETTLE_END で PEAK→1.0、以降 1.0
        let scale = if progress < CHAIN_POP_END {
            (progress / CHAIN_POP_END) * CHAIN_SCALE_PEAK
        } else if progress < CHAIN_SETTLE_END {
            let t = (progress - CHAIN_POP_END) / (CHAIN_SETTLE_END - CHAIN_POP_END);
            CHAIN_SCALE_PEAK - t * (CHAIN_SCALE_PEAK - 1.0)
        } else {
            1.0
        };
        let alpha = if progress < CHAIN_FADE_START {
            1.0
        } else {
            (1.0 - (progress - CHAIN_FADE_START) / (1.0 - CHAIN_FADE_START)).max(0.0)
        };
        let float_dr = -progress * CHAIN_FLOAT_DR;

        let text = localization::format_chain(eff.count, self.lang);
        let font_size = ((CHAIN_FONT as f32) * scale).max(1.0) as u16;
        let dims = measure_text(&text, Some(&self.font), font_size, 1.0);
        let cx = self.field_x + (eff.col + 0.5) * PUYO_SIZE;
        let cy = self.field_y + (eff.row + 0.5 + float_dr) * PUYO_SIZE;
        let x = cx - dims.width / 2.0;
        let y = cy + dims.height / 3.0;

        let body = Color {
            a: alpha,
            ..TEXT_HIGHLIGHT_COLOR
        };
        let outline = Color {
            a: alpha,
            ..TEXT_OUTLINE_COLOR
        };
        self.draw_outlined_text(&text, x, y, font_size, body, outline, CHAIN_OUTLINE);
    }

    pub fn draw_score(&self, score: u32) {
        let text = format!("{score:08}");
        let field_w = PUYO_SIZE * self.cols as f32;
        let dims = measure_text(&text, Some(&self.font), SCORE_FONT, 1.0);
        let field_h = PUYO_SIZE * self.rows as f32;
        let field_bottom = self.field_y + field_h;
        let bg_bottom = field_bottom + FIELD_PADDING * 3.0;
        let x = self.field_x + field_w / 2.0 - dims.width / 2.0;
        let y = (field_bottom + bg_bottom) / 2.0 + dims.height / 3.0;

        self.draw_outlined_text(
            &text,
            x,
            y,
            SCORE_FONT,
            TEXT_HIGHLIGHT_COLOR,
            TEXT_OUTLINE_COLOR,
            SCORE_OUTLINE,
        );
    }

    /// 設定画面を描画
    pub fn draw_settings(
        &self,
        puyo_colors: usize,
        bgm_volume: f32,
        se_volume: f32,
        showing_credits: bool,
        bgm_playing: bool,
        focused_index: usize,
        showing_language_picker: bool,
        lang_picker_index: usize,
    ) {
        // パネル領域 (画面中央)
        let panel_x = (self.window_width - PANEL_W) / 2.0;
        let panel_y = (self.window_height - PANEL_H) / 2.0;
        let panel_cx = panel_x + PANEL_W / 2.0;

        // パネル背景
        draw_rectangle(panel_x, panel_y, PANEL_W, PANEL_H, PANEL_BG);
        draw_rectangle_lines(panel_x, panel_y, PANEL_W, PANEL_H, PANEL_BORDER, WHITE);

        let s = self.s();
        if showing_credits {
            // ===== Credits 画面 =====
            self.draw_text_anchored(
                s.credits_title,
                panel_cx,
                panel_y + PANEL_TITLE_DY,
                PANEL_TITLE_FONT,
                WHITE,
                TextAlign::Center,
            );

            let cat_x = panel_x + CRED_CAT_DX;
            let val_x = panel_x + CRED_VAL_DX;
            let bgm_y = panel_y + CRED_BGM_ROW_DY;
            let se_y = panel_y + CRED_SE_ROW_DY;

            self.draw_text_anchored(
                s.credits_bgm_label,
                cat_x,
                bgm_y,
                CRED_CAT_FONT,
                CRED_CAT_COLOR,
                TextAlign::Left,
            );
            self.draw_text_anchored(
                "ニコニコモンズ: nc148246",
                val_x,
                bgm_y,
                CRED_VAL_FONT,
                WHITE,
                TextAlign::Left,
            );
            self.draw_text_anchored(
                s.credits_se_label,
                cat_x,
                se_y,
                CRED_CAT_FONT,
                CRED_CAT_COLOR,
                TextAlign::Left,
            );
            self.draw_text_anchored(
                "ニコニコモンズ: nc268086,nc168010,nc389893",
                val_x,
                se_y,
                CRED_VAL_FONT,
                WHITE,
                TextAlign::Left,
            );

            // Back ボタン (Credits 画面では index 0 だけ focusable)
            let back_rect = Rect::new(
                panel_cx - CRED_BACK_W / 2.0,
                panel_y + PANEL_H - BACK_BTN_BOTTOM_GAP,
                CRED_BACK_W,
                CRED_BACK_H,
            );
            self.draw_panel_button(back_rect, s.settings_back, VALUE_FONT, focused_index == 0);
        } else if showing_language_picker {
            // ===== Language ピッカー =====
            self.draw_text_anchored(
                s.settings_language,
                panel_cx,
                panel_y + PANEL_TITLE_DY,
                PANEL_TITLE_FONT,
                WHITE,
                TextAlign::Center,
            );
            // ピッカー用ヒント (Navigate + Enter)
            self.draw_text_anchored(
                &format!(
                    "{}    {}: {}",
                    s.hint_navigate, s.action_close, s.hint_enter_kw
                ),
                panel_cx,
                panel_y + HINT_LINE2_DY,
                HINT_FONT,
                HINT_COLOR,
                TextAlign::Center,
            );

            // 2 択を縦に並べる (English / 日本語)
            let labels = [Lang::En.display_name(), Lang::Ja.display_name()];
            let row_top = panel_y + SLIDER_TOP_DY + SLIDER_ROW_H;
            for (i, label) in labels.iter().enumerate() {
                let y = row_top + (i as f32) * SLIDER_ROW_H;
                let focused = i == lang_picker_index;
                let color = if focused { TEXT_HIGHLIGHT_COLOR } else { WHITE };
                self.draw_text_anchored(label, panel_cx, y, VALUE_FONT, color, TextAlign::Center);
                if focused {
                    let dim = measure_text(label, Some(&self.font), VALUE_FONT, 1.0);
                    let label_x = panel_cx - dim.width / 2.0;
                    self.draw_text_anchored(
                        ">",
                        label_x - FOCUS_MARKER_GAP,
                        y,
                        VALUE_FONT,
                        TEXT_HIGHLIGHT_COLOR,
                        TextAlign::Left,
                    );
                }
            }
        } else {
            // ===== 設定画面 =====
            self.draw_text_anchored(
                s.settings_title,
                panel_cx,
                panel_y + PANEL_TITLE_DY,
                PANEL_TITLE_FONT,
                WHITE,
                TextAlign::Center,
            );

            // 動的ヒント: focus に応じて Adjust / Action 部分を出し分ける
            let action_label = match focused_index {
                1 => Some(if bgm_playing {
                    s.action_stop_bgm
                } else {
                    s.action_test_bgm
                }),
                2 => Some(s.action_test_se),
                3 => Some(s.action_choose_language),
                4 => Some(s.action_show_credits),
                5 => Some(s.action_close),
                _ => None, // Puyo colors は Enter で何もしない
            };
            let adjustable = matches!(focused_index, 0 | 1 | 2);

            let line1 = if adjustable {
                format!("{}    {}", s.hint_navigate, s.hint_adjust)
            } else {
                s.hint_navigate.to_string()
            };
            let line2 = if let Some(action) = action_label {
                format!("{}: {}    {}", action, s.hint_enter_kw, s.hint_back)
            } else {
                s.hint_back.to_string()
            };
            self.draw_text_anchored(
                &line1,
                panel_cx,
                panel_y + HINT_LINE1_DY,
                HINT_FONT,
                HINT_COLOR,
                TextAlign::Center,
            );
            self.draw_text_anchored(
                &line2,
                panel_cx,
                panel_y + HINT_LINE2_DY,
                HINT_FONT,
                HINT_COLOR,
                TextAlign::Center,
            );

            let slider_label_x = panel_x + SLIDER_LABEL_DX;
            let slider_bar_x = panel_x + SLIDER_BAR_DX;
            let slider_value_x = panel_x + SLIDER_VALUE_DX;
            let slider_top = panel_y + SLIDER_TOP_DY;
            let focus_marker_x = slider_label_x - FOCUS_MARKER_GAP;
            let row_y = |i: usize| slider_top + SLIDER_ROW_H * (i as f32 + 0.5);

            // Puyo colors (focus index 0)
            let y0 = row_y(0);
            self.draw_focus_marker(focus_marker_x, y0, focused_index == 0);
            self.draw_text_anchored(
                s.settings_puyo_colors,
                slider_label_x,
                y0,
                VALUE_FONT,
                focus_color(focused_index == 0),
                TextAlign::Left,
            );
            self.draw_panel_slider(
                slider_bar_x,
                y0,
                SLIDER_BAR_W,
                puyo_colors as f32,
                3.0,
                5.0,
                focused_index == 0,
            );
            self.draw_text_anchored(
                &format!("{}", puyo_colors),
                slider_value_x,
                y0,
                VALUE_FONT,
                WHITE,
                TextAlign::Left,
            );

            // BGM volume (focus index 1) - Enter で BGM テスト toggle (hint で示す)
            let y1 = row_y(1);
            self.draw_focus_marker(focus_marker_x, y1, focused_index == 1);
            self.draw_text_anchored(
                s.settings_bgm_volume,
                slider_label_x,
                y1,
                VALUE_FONT,
                focus_color(focused_index == 1),
                TextAlign::Left,
            );
            self.draw_panel_slider(
                slider_bar_x,
                y1,
                SLIDER_BAR_W,
                bgm_volume,
                0.0,
                1.0,
                focused_index == 1,
            );
            self.draw_text_anchored(
                &format!("{:.2}", bgm_volume),
                slider_value_x,
                y1,
                VALUE_FONT,
                WHITE,
                TextAlign::Left,
            );

            // SE volume (focus index 2) - Enter で SE 1 回再生
            let y2 = row_y(2);
            self.draw_focus_marker(focus_marker_x, y2, focused_index == 2);
            self.draw_text_anchored(
                s.settings_se_volume,
                slider_label_x,
                y2,
                VALUE_FONT,
                focus_color(focused_index == 2),
                TextAlign::Left,
            );
            self.draw_panel_slider(
                slider_bar_x,
                y2,
                SLIDER_BAR_W,
                se_volume,
                0.0,
                1.0,
                focused_index == 2,
            );
            self.draw_text_anchored(
                &format!("{:.2}", se_volume),
                slider_value_x,
                y2,
                VALUE_FONT,
                WHITE,
                TextAlign::Left,
            );

            // Language (focus index 3) - 選択式 (Enter でピッカーを開く)
            let y3 = row_y(3);
            self.draw_focus_marker(focus_marker_x, y3, focused_index == 3);
            self.draw_text_anchored(
                s.settings_language,
                slider_label_x,
                y3,
                VALUE_FONT,
                focus_color(focused_index == 3),
                TextAlign::Left,
            );
            let lang_value_x = (slider_bar_x + slider_value_x) / 2.0;
            self.draw_text_anchored(
                self.lang.display_name(),
                lang_value_x,
                y3,
                VALUE_FONT,
                WHITE,
                TextAlign::Center,
            );

            // Credits link (focus index 4)
            let credits_y = row_y(4);
            self.draw_panel_link(
                s.settings_credits,
                panel_cx,
                credits_y,
                VALUE_FONT,
                focused_index == 4,
            );

            // Back button (focus index 5)
            let close_rect = Rect::new(
                panel_cx - BACK_BTN_W / 2.0,
                panel_y + PANEL_H - BACK_BTN_BOTTOM_GAP,
                BACK_BTN_W,
                BACK_BTN_H,
            );
            self.draw_panel_button(close_rect, s.settings_back, VALUE_FONT, focused_index == 5);
        }
    }

    // ===== UI プリミティブ (pure draw) =====

    /// 指定位置にテキストを描画 (alignment 指定可)
    fn draw_text_anchored(
        &self,
        text: &str,
        x: f32,
        y_center: f32,
        font_size: u16,
        color: Color,
        align: TextAlign,
    ) {
        let dim = measure_text(text, Some(&self.font), font_size, 1.0);
        let draw_x = match align {
            TextAlign::Left => x,
            TextAlign::Center => x - dim.width / 2.0,
        };
        draw_text_ex(
            text,
            draw_x.round(),
            (y_center + dim.height / 2.0).round(),
            TextParams {
                font: Some(&self.font),
                font_size,
                color,
                ..Default::default()
            },
        );
    }

    /// 左側に focus マーカーを描画
    fn draw_focus_marker(&self, x: f32, y_center: f32, focused: bool) {
        if focused {
            // ASCII '>' は label と同じ baseline で揃う
            self.draw_text_anchored(">", x, y_center, 16, YELLOW, TextAlign::Left);
        }
    }

    /// スライダー描画 (input 処理なし)
    fn draw_panel_slider(
        &self,
        x: f32,
        y_center: f32,
        width: f32,
        value: f32,
        min: f32,
        max: f32,
        focused: bool,
    ) {
        let bar_h = 6.0;
        let knob_r = 8.0;
        let bar_y = y_center - bar_h / 2.0;

        // バー背景
        draw_rectangle(x, bar_y, width, bar_h, Color::new(0.3, 0.3, 0.3, 1.0));
        // 進捗
        let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
        let fill_w = width * t;
        let fill_color = if focused {
            Color::new(0.9, 0.9, 0.5, 1.0)
        } else {
            Color::new(0.7, 0.7, 0.85, 1.0)
        };
        draw_rectangle(x, bar_y, fill_w, bar_h, fill_color);
        // ノブ
        let knob_x = x + fill_w;
        let knob_color = if focused {
            Color::new(1.0, 1.0, 0.6, 1.0)
        } else {
            Color::new(0.9, 0.9, 0.9, 1.0)
        };
        draw_circle(knob_x, y_center, knob_r, knob_color);
    }

    /// ボタン描画 (input 処理なし)
    fn draw_panel_button(&self, rect: Rect, label: &str, font_size: u16, focused: bool) {
        let bg = if focused {
            Color::new(0.4, 0.35, 0.15, 1.0)
        } else {
            Color::new(0.2, 0.2, 0.25, 1.0)
        };
        let border = if focused {
            YELLOW
        } else {
            Color::new(0.5, 0.5, 0.5, 1.0)
        };
        let border_w = if focused { 2.0 } else { 1.5 };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, border_w, border);
        self.draw_text_anchored(
            label,
            rect.x + rect.w / 2.0,
            rect.y + rect.h / 2.0,
            font_size,
            WHITE,
            TextAlign::Center,
        );
    }

    /// テキストリンク描画 (input 処理なし)
    fn draw_panel_link(&self, text: &str, cx: f32, cy: f32, font_size: u16, focused: bool) {
        let dim = measure_text(text, Some(&self.font), font_size, 1.0);
        let color = if focused {
            YELLOW
        } else {
            Color::new(0.65, 0.65, 0.65, 1.0)
        };
        self.draw_text_anchored(text, cx, cy, font_size, color, TextAlign::Center);
        let underline_x = cx - dim.width / 2.0;
        draw_line(
            underline_x,
            cy + dim.height / 2.0 + 2.0,
            underline_x + dim.width,
            cy + dim.height / 2.0 + 2.0,
            1.0,
            color,
        );
    }
}

/// focus 状態に応じたテキスト色
fn focus_color(focused: bool) -> Color {
    if focused { YELLOW } else { WHITE }
}

/// テキストの水平アラインメント
enum TextAlign {
    Left,
    Center,
}
