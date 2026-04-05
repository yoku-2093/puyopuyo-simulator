use crate::constants::*;
use crate::puyo::*;
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct Renderer {
    textures: HashMap<PuyoColor, Texture2D>,
    background: Texture2D,
    field_bg: Texture2D,
    field: Texture2D,
}

impl Renderer {
    pub async fn new() -> Self {
        let colors = [
            (PuyoColor::Blue, "assets/images/puyo/blue.png"),
            (PuyoColor::Green, "assets/images/puyo/green.png"),
            (PuyoColor::Red, "assets/images/puyo/red.png"),
            (PuyoColor::Yellow, "assets/images/puyo/yellow.png"),
            (PuyoColor::Purple, "assets/images/puyo/purple.png"),
        ];

        let mut textures = HashMap::new();
        for (color, path) in colors {
            let texture = load_texture(path).await.unwrap();
            texture.set_filter(FilterMode::Nearest);
            textures.insert(color, texture);
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

        Renderer {
            textures,
            background,
            field_bg,
            field,
        }
    }

    pub fn draw_title(&self) {
        draw_text("Hello, PuyoPuyo Simulator!", 20.0, 20.0, 30.0, BLACK);
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
    fn draw_centered_text(&self, text: &str, font_size: f32, color: Color) {
        let field_w = PUYO_SIZE * COLS as f32;
        let field_h = PUYO_SIZE * ROWS as f32;
        let center_x = FIELD_X + field_w / 2.0;
        let center_y = FIELD_Y + field_h / 2.0;

        let dimensions = measure_text(text, None, font_size as u16, 1.0);
        draw_text(
            text,
            center_x - dimensions.width / 2.0,
            center_y + dimensions.height / 2.0,
            font_size,
            color,
        );
    }

    /// スタート画面の描画
    pub fn draw_press_start(&self) {
        let alpha = ((get_time() * 3.0).sin() * 0.5 + 0.5) as f32;
        self.draw_centered_text("PRESS ENTER or SPACE", 36.0, Color::new(1.0, 1.0, 0.0, alpha));
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
        let alpha = ((get_time() * 3.0).sin() * 0.5 + 0.5) as f32;
        self.draw_centered_text("ばたんきゅ〜", 40.0, Color::new(1.0, 0.3, 0.3, alpha));
    }

    pub fn draw_puyo(&self, color: PuyoColor, col: usize, row: usize) {
        assert!(col < COLS, "col out of range: {} (max {})", col, COLS - 1);
        assert!(row < ROWS, "row out of range: {} (max {})", row, ROWS - 1);
        draw_texture_ex(
            &self.textures[&color],
            FIELD_X + col as f32 * PUYO_SIZE,
            FIELD_Y + row as f32 * PUYO_SIZE,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(PUYO_SIZE, PUYO_SIZE)),
                ..Default::default()
            },
        );
    }
}
