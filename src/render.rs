use crate::constants::*;
use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuyoColor {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}

pub struct Renderer {
    textures: HashMap<PuyoColor, Texture2D>,
    background: Texture2D,
    field_bg: Texture2D,
    field: Texture2D,
}

impl Renderer {
    // フィールド左上のピクセル座標（ウィンドウ中央に配置）
    const FIELD_X: f32 = (WINDOW_WIDTH - PUYO_SIZE * COLS as f32) / 2.0;
    const FIELD_Y: f32 = (WINDOW_HEIGHT - PUYO_SIZE * ROWS as f32) / 2.0;

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

        let background = load_texture("assets/images/background/window.png").await.unwrap();
        let field_bg = load_texture("assets/images/background/field_bg.png")
            .await
            .unwrap();
        let field = load_texture("assets/images/background/field.png").await.unwrap();

        Renderer {
            textures,
            background,
            field_bg,
            field,
        }
    }

    pub fn draw_puyo(&self, color: PuyoColor, col: usize, row: usize) {
        assert!(col < COLS, "col out of range: {} (max {})", col, COLS - 1);
        assert!(row < ROWS, "row out of range: {} (max {})", row, ROWS - 1);
        draw_texture_ex(
            &self.textures[&color],
            Self::FIELD_X + col as f32 * PUYO_SIZE,
            Self::FIELD_Y + row as f32 * PUYO_SIZE,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(PUYO_SIZE, PUYO_SIZE)),
                ..Default::default()
            },
        );
    }

    pub fn draw_background(&self) {
        clear_background(BLACK);
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
            Self::FIELD_X - padding,
            Self::FIELD_Y - padding,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(bg_w, bg_h)),
                ..Default::default()
            },
        );

        // フィールド本体（field）を中央に描画
        draw_texture_ex(
            &self.field,
            Self::FIELD_X,
            Self::FIELD_Y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(field_w, field_h)),
                ..Default::default()
            },
        );
    }
}
