use crate::types::Puyo;
use egui_macroquad::egui;
use macroquad::prelude::*;
use std::collections::HashMap;

const PUYO_SIZE: f32 = 60.0; // ぷよ1個あたりの描画サイズ（ピクセル）
const FIELD_PADDING: f32 = 20.0; // フィールド外枠の余白（ピクセル）
const NEXT_ANIM_DURATION: f64 = 0.15; // ネクスト遷移アニメーションの長さ（秒）

const JAPANESE_FONT: &[u8] = include_bytes!("../assets/fonts/Hiragino_Sans_W6.ttf");

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

        // 日本語フォント。include_bytes! でバイナリに埋め込み、macroquad と egui で共有
        let font = load_ttf_font_from_bytes(JAPANESE_FONT).unwrap();
        egui_macroquad::cfg(|ctx| Self::install_egui_japanese_font(ctx));

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

    fn install_egui_japanese_font(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "japanese".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(JAPANESE_FONT)),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, "japanese".to_owned());
        }
        ctx.set_fonts(fonts);
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
        self.draw_centered_text(
            "Press S for Settings",
            16.0,
            Color::new(1.0, 1.0, 1.0, 0.7),
            40.0,
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

    /// 設定画面を描画。
    /// 同フレームで他の egui 関数を呼ばないこと（ui() が上書きされるため）。
    pub fn draw_settings(
        &self,
        puyo_colors: &mut usize,
        bgm_volume: &mut f32,
        se_volume: &mut f32,
        showing_credits: &mut bool,
    ) -> SettingsResult {
        let mut result = SettingsResult::default();
        let credits = *showing_credits;

        egui_macroquad::ui(|ctx| {
            egui::Window::new(if credits { "Credits" } else { "Settings" })
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .frame(
                    egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::symmetric(30, 22)),
                )
                .show(ctx, |ui| {
                    ui.spacing_mut().slider_width = 160.0;
                    if credits {
                        let category = |text: &str| {
                            egui::RichText::new(text)
                                .size(13.0)
                                .color(egui::Color32::from_gray(150))
                        };
                        let value = |text: &str| egui::RichText::new(text).size(14.0);

                        egui::Grid::new("credits_grid")
                            .num_columns(2)
                            .spacing([20.0, 14.0])
                            .show(ui, |ui| {
                                ui.label(category("BGM"));
                                ui.label(value("ニコニコモンズ: nc148246"));
                                ui.end_row();

                                ui.label(category("SE"));
                                ui.label(value("ニコニコモンズ: nc268086"));
                                ui.end_row();
                            });
                        ui.add_space(28.0);
                        ui.vertical_centered(|ui| {
                            let back = egui::Button::new(egui::RichText::new("Back").size(13.0))
                                .min_size(egui::vec2(120.0, 30.0));
                            if ui.add(back).clicked() {
                                *showing_credits = false;
                            }
                        });
                    } else {
                        egui::Grid::new("settings_grid")
                            .num_columns(3)
                            .spacing([16.0, 14.0])
                            .show(ui, |ui| {
                                ui.label("Puyo colors");
                                ui.add(egui::Slider::new(puyo_colors, 3..=5).show_value(false));
                                ui.label(format!("{}", *puyo_colors));
                                ui.end_row();

                                ui.label("BGM volume");
                                ui.add(egui::Slider::new(bgm_volume, 0.0..=1.0).show_value(false));
                                ui.label(format!("{:.2}", *bgm_volume));
                                ui.end_row();

                                ui.label("SE volume");
                                let se_resp = ui
                                    .add(egui::Slider::new(se_volume, 0.0..=1.0).show_value(false));
                                if se_resp.drag_stopped() {
                                    result.test_se = true;
                                }
                                ui.label(format!("{:.2}", *se_volume));
                                ui.end_row();
                            });
                        ui.add_space(24.0);
                        ui.vertical_centered(|ui| {
                            // Credits: テキストのみ。hover で色を明るくして「太くなった」感を出す
                            let hover_id = egui::Id::new("credits_link_hover");
                            let was_hovered =
                                ui.data(|d| d.get_temp::<bool>(hover_id).unwrap_or(false));
                            let credits_text = egui::RichText::new("Credits")
                                .size(13.0)
                                .underline()
                                .color(if was_hovered {
                                    egui::Color32::from_gray(230)
                                } else {
                                    egui::Color32::from_gray(150)
                                })
                                .strong();
                            let credits_resp =
                                ui.add(egui::Button::new(credits_text).frame(false));
                            ui.data_mut(|d| d.insert_temp(hover_id, credits_resp.hovered()));
                            if credits_resp.clicked() {
                                *showing_credits = true;
                            }
                            ui.add_space(14.0);
                            // Close: 控えめサイズで一番下
                            let close_btn =
                                egui::Button::new(egui::RichText::new("Close (ESC)").size(13.0))
                                    .min_size(egui::vec2(120.0, 30.0));
                            if ui.add(close_btn).clicked() {
                                result.close = true;
                            }
                        });
                    }
                });
        });
        egui_macroquad::draw();
        result
    }
}

/// 設定画面から発生したイベント
#[derive(Default)]
pub struct SettingsResult {
    pub close: bool,
    pub test_se: bool,
}
