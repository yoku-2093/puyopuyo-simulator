use crate::constants::*;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct Renderer {
    textures: HashMap<Puyo, Texture2D>,
    background: Texture2D,
    field_bg: Texture2D,
    field: Texture2D,
    font: Font,
    game_over_text: Texture2D,
}

impl Renderer {
    pub async fn new() -> Self {
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

        Renderer {
            textures,
            background,
            field_bg,
            field,
            font,
            game_over_text,
        }
    }

    pub fn draw_background(&self) {
        draw_texture_ex(
            &self.background,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
                ..Default::default()
            },
        );
    }

    pub fn draw_field(&self) {
        let field_w = PUYO_SIZE * COLS as f32;
        let field_h = PUYO_SIZE * ROWS as f32;
        let padding = 20.0;
        let bg_w = field_w + padding * 2.0;
        let bg_h = field_h + padding * 2.0;

        // 外枠（field_bg）をフィールドより一回り大きく描画
        draw_texture_ex(
            &self.field_bg,
            FIELD_X - padding,
            FIELD_Y - padding,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(bg_w, bg_h)),
                ..Default::default()
            },
        );

        // フィールド本体（field）を中央に描画
        draw_texture_ex(
            &self.field,
            FIELD_X,
            FIELD_Y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(field_w, field_h)),
                ..Default::default()
            },
        );
    }

    /// フィールド中央にテキストを描画するヘルパー
    fn draw_centered_text(&self, text: &str, font_size: f32, color: Color, y_offset: f32) {
        let field_w = PUYO_SIZE * COLS as f32;
        let field_h = PUYO_SIZE * ROWS as f32;
        let center_x = FIELD_X + field_w / 2.0;
        let center_y = FIELD_Y + field_h / 2.0 + y_offset;

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
        let field_w = PUYO_SIZE * COLS as f32;
        let field_h = PUYO_SIZE * ROWS as f32;
        // 半透明の暗幕
        draw_rectangle(
            FIELD_X,
            FIELD_Y,
            field_w,
            field_h,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        // テクスチャをスケールアニメーションで描画
        let t = ((get_time() * 1.5).sin() * 0.5 + 0.5) as f32;
        let scale = 0.45 + 0.1 * t; // 0.45〜0.55
        let tex_w = self.game_over_text.width() * scale;
        let tex_h = self.game_over_text.height() * scale;
        let field_w2 = PUYO_SIZE * COLS as f32;
        let field_h2 = PUYO_SIZE * ROWS as f32;
        let cx = FIELD_X + field_w2 / 2.0 - tex_w / 2.0;
        let cy = FIELD_Y + field_h2 / 2.0 - tex_h / 2.0;
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
        let x = FIELD_X + col * PUYO_SIZE - (w - PUYO_SIZE) / 2.0;
        let y = FIELD_Y + row * PUYO_SIZE + (PUYO_SIZE - h);
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
}
