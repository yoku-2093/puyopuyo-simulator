use crate::types::Puyo;
use macroquad::prelude::*;
use std::collections::HashMap;

const PUYO_SIZE: f32 = 60.0; // ぷよ1個あたりの描画サイズ（ピクセル）
const FIELD_PADDING: f32 = 20.0; // フィールド外枠の余白（ピクセル）
const NEXT_ANIM_DURATION: f64 = 0.15; // ネクスト遷移アニメーションの長さ（秒）

pub struct NextPuyo {
    pub axis: Puyo,
    pub child: Puyo,
    pub generation: u32, // 世代（スポーンごとにインクリメント）
}

impl NextPuyo {
    pub fn new(axis: Puyo, child: Puyo, generation: u32) -> Self {
        NextPuyo { axis, child, generation }
    }
}

struct NextAnim {
    start_time: f64,                  // アニメーション開始時刻
    prev_generation: u32,             // 前回の世代（変化検出用）
    current: Option<(Puyo, Puyo)>,    // 現在のネクスト
    exiting: Option<(Puyo, Puyo)>,    // 上に出ていく旧ネクスト
}

impl NextAnim {
    fn new() -> Self {
        NextAnim {
            start_time: 0.0,
            prev_generation: 0,
            current: None,
            exiting: None,
        }
    }
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

        let font = load_ttf_font("assets/fonts/Hiragino_Sans_W6.ttf")
            .await
            .unwrap();

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
        }
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
            center_x - dimensions.width / 2.0,
            center_y + dimensions.height / 2.0,
            params,
        );
    }

    /// スタート画面の描画
    pub fn draw_press_start(&self) {
        let t = ((get_time() * 0.7 % 1.0) as f32 * 2.0 - 1.0).abs();
        let alpha = 0.4 + 0.6 * t * t * (3.0 - 2.0 * t);
        self.draw_centered_text(
            "PRESS ENTER or SPACE",
            24.0,
            Color::new(1.0, 1.0, 0.0, alpha),
            0.0,
        );
    }

    /// ゲームオーバー画面の描画
    pub fn draw_game_over(&self) {
        let field_w = PUYO_SIZE * self.cols as f32;
        let field_h = PUYO_SIZE * self.rows as f32;
        // 半透明の暗幕
        draw_rectangle(
            self.field_x,
            self.field_y,
            field_w,
            field_h,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        // テクスチャをスケールアニメーションで描画
        let t = ((get_time() * 1.5).sin() * 0.5 + 0.5) as f32;
        let scale = 0.45 + 0.1 * t; // 0.45〜0.55
        let tex_w = self.game_over_text.width() * scale;
        let tex_h = self.game_over_text.height() * scale;
        let field_w2 = PUYO_SIZE * self.cols as f32;
        let field_h2 = PUYO_SIZE * self.rows as f32;
        let cx = self.field_x + field_w2 / 2.0 - tex_w / 2.0;
        let cy = self.field_y + field_h2 / 2.0 - tex_h / 2.0;
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
        self.draw_centered_text(
            "Press ESC to return",
            16.0,
            Color::new(1.0, 1.0, 1.0, 0.8),
            80.0,
        );
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

    /// ネクストエリアの背景を描画（常時表示）
    pub fn draw_next_area(&self) {
        let field_w = PUYO_SIZE * self.cols as f32;
        let gap = 5.0;
        let area_w = PUYO_SIZE * 2.5;
        let area_h = PUYO_SIZE * 5.5;
        let area_x = self.field_x + field_w + FIELD_PADDING + gap;
        let area_y = self.field_y - FIELD_PADDING;

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
    pub fn draw_next_puyos(&mut self, next: &NextPuyo, next_next: &NextPuyo) {
        let now = get_time();

        // 世代が変わったらアニメーション開始
        if self.next_anim.prev_generation != next.generation {
            self.next_anim.exiting = self.next_anim.current;
            self.next_anim.start_time = now;
            self.next_anim.prev_generation = next.generation;
        }
        self.next_anim.current = Some((next.axis, next.child));

        let field_w = PUYO_SIZE * self.cols as f32;
        let gap = 5.0;
        let area_w = PUYO_SIZE * 2.5;
        let area_h = PUYO_SIZE * 5.5;
        let area_x = self.field_x + field_w + FIELD_PADDING + gap;
        let area_y = self.field_y - FIELD_PADDING;

        // アニメーション進行度（0.0→1.0、ease-out）
        let raw_t = ((now - self.next_anim.start_time) / NEXT_ANIM_DURATION).clamp(0.0, 1.0) as f32;
        let t = 1.0 - (1.0 - raw_t) * (1.0 - raw_t); // ease-out quadratic

        // クリッピング範囲（白枠のさらに内側）
        let border = 18.0;
        let clip_top = area_y + border;
        let clip_bottom = area_y + area_h - border;

        // ネクスト安定位置（左寄り）
        let next_rest_x = area_x + PUYO_SIZE * 0.3;
        let next_rest_y = area_y + PUYO_SIZE * 0.5;
        // ネクネク安定位置（右寄り）
        let nn_scale = 0.75;
        let nn_size = PUYO_SIZE * nn_scale;
        let nn_rest_x = area_x + area_w - nn_size - PUYO_SIZE * 0.3;
        let nn_rest_y = area_y + PUYO_SIZE * 3.2;
        // 入口（右下）・出口（左上）
        let entry_x = nn_rest_x;
        let entry_y = clip_bottom + nn_size * 2.0; // ぷよ2個分下から（クリッピングで隠れる）
        let exit_y = area_y - PUYO_SIZE * 2.5;

        // 退出中の旧ネクスト: ネクスト位置 → 左上に出ていく
        if let Some(exiting) = self.next_anim.exiting {
            let ex_y = next_rest_y + (exit_y - next_rest_y) * t;
            if ex_y + PUYO_SIZE * 2.0 > clip_top {
                self.draw_puyo_clipped(exiting.1, next_rest_x, ex_y, 1.0, clip_top, clip_bottom);
                self.draw_puyo_clipped(exiting.0, next_rest_x, ex_y + PUYO_SIZE, 1.0, clip_top, clip_bottom);
            }
        }

        // ネクスト: 右下のネクネク位置 → 左上のネクスト位置にスライド
        let next_x = nn_rest_x + (next_rest_x - nn_rest_x) * t;
        let next_y = nn_rest_y + (next_rest_y - nn_rest_y) * t;
        // スライド中はネクネクサイズ → ネクストサイズに拡大
        let next_scale = nn_scale + (1.0 - nn_scale) * t;
        let next_size = PUYO_SIZE * next_scale;
        self.draw_puyo_clipped(next.child, next_x, next_y, next_scale, clip_top, clip_bottom);
        self.draw_puyo_clipped(next.axis, next_x, next_y + next_size, next_scale, clip_top, clip_bottom);

        // ネクネク: 右下から → ネクネク位置にスライド
        let nn_x = entry_x + (nn_rest_x - entry_x) * t;
        let nn_y = entry_y + (nn_rest_y - entry_y) * t;
        self.draw_puyo_clipped(next_next.child, nn_x, nn_y, nn_scale, clip_top, clip_bottom);
        self.draw_puyo_clipped(next_next.axis, nn_x, nn_y + nn_size, nn_scale, clip_top, clip_bottom);
    }

    /// エリア範囲内のみ描画するぷよ（はみ出し部分は非表示）
    fn draw_puyo_clipped(&self, puyo: Puyo, x: f32, y: f32, scale: f32, clip_top: f32, clip_bottom: f32) {
        let size = PUYO_SIZE * scale;
        if y + size <= clip_top || y >= clip_bottom {
            return; // 完全にエリア外
        }
        // エリア内に収まる部分だけ描画
        let visible_top = (clip_top - y).max(0.0);
        let visible_bottom = (clip_bottom - y).min(size);
        let src_top = visible_top / size;
        let src_bottom = visible_bottom / size;
        draw_texture_ex(
            &self.textures[&puyo],
            x,
            y + visible_top,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(size, visible_bottom - visible_top)),
                source: Some(Rect::new(
                    0.0,
                    src_top * self.textures[&puyo].height(),
                    self.textures[&puyo].width(),
                    (src_bottom - src_top) * self.textures[&puyo].height(),
                )),
                ..Default::default()
            },
        );
    }

    pub fn draw_particle(&self, col: f32, row: f32, size: f32, color: Color) {
        let x = self.field_x + col * PUYO_SIZE + PUYO_SIZE / 2.0;
        let y = self.field_y + row * PUYO_SIZE + PUYO_SIZE / 2.0;
        let r = size * PUYO_SIZE;
        draw_circle(x, y, r, color);
    }

    pub fn draw_score(&self, score: u32) {
        let text = format!("{score:08}");
        let font_size = 36.0;
        let field_w = PUYO_SIZE * self.cols as f32;
        let dims = measure_text(&text, Some(&self.font), font_size as u16, 1.0);
        let field_h = PUYO_SIZE * self.rows as f32;
        let padding = FIELD_PADDING;
        let field_bottom = self.field_y + field_h;
        let bg_bottom = field_bottom + padding * 3.0;
        let x = self.field_x + field_w / 2.0 - dims.width / 2.0;
        let y = (field_bottom + bg_bottom) / 2.0 + dims.height / 3.0;

        // 縁取り（8方向にずらして黒で描画）
        let outline = 2.0;
        for (dx, dy) in [
            (-outline, 0.0),
            (outline, 0.0),
            (0.0, -outline),
            (0.0, outline),
            (-outline, -outline),
            (outline, -outline),
            (-outline, outline),
            (outline, outline),
        ] {
            draw_text_ex(
                &text,
                x + dx,
                y + dy,
                TextParams {
                    font: Some(&self.font),
                    font_size: font_size as u16,
                    color: Color::new(0.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                },
            );
        }

        // 本体（黄色）
        draw_text_ex(
            &text,
            x,
            y,
            TextParams {
                font: Some(&self.font),
                font_size: font_size as u16,
                color: Color::new(1.0, 1.0, 0.3, 1.0),
                ..Default::default()
            },
        );
    }
}
