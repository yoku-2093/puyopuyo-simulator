# egui-macroquad ガイド

macroquad の上で egui（Immediate Mode GUI）を使うためのライブラリ。スコア表示、設定画面、デバッグパネルなどのテキスト/ウィジェット系 UI を簡単に作れる。

参考:
- 公式 docs: https://docs.rs/egui-macroquad/
- egui docs: https://docs.rs/egui/

---

## セットアップ

`Cargo.toml`:
```toml
[dependencies]
macroquad = "0.4"
egui-macroquad = "0.17"
```

---

## 基本構造

毎フレーム以下の3つを順番に呼ぶ:

```rust
loop {
    // ① macroquad の通常描画（背景、ゲームオブジェクト等）
    draw_texture_ex(...);
    draw_circle(...);

    // ② egui の UI 定義（毎フレーム宣言する）
    egui_macroquad::ui(|egui_ctx| {
        egui::Window::new("My Window").show(egui_ctx, |ui| {
            ui.label("Hello");
        });
    });

    // ③ egui を画面に描画（必ず最後）
    egui_macroquad::draw();

    next_frame().await;
}
```

**重要**: `ui()` で UI を「宣言」し、`draw()` で実際に画面に描画する。`draw()` を呼ばないと表示されない。

---

## API 一覧

| 関数 | 役割 |
|---|---|
| `egui_macroquad::ui(\|ctx\| ...)` | UI を毎フレーム宣言。クロージャで `egui::Context` を受け取る |
| `egui_macroquad::draw()` | 宣言済みの UI を画面に描画する |
| `egui_macroquad::cfg(\|ctx\| ...)` | フレーム外で egui の設定を行う |

---

## ウィンドウ系

### Window（タイトルバー付き、ドラッグ移動可）

```rust
egui::Window::new("設定")
    .collapsible(false)        // 折りたたみ無効
    .resizable(false)          // リサイズ無効
    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))  // 中央固定
    .show(egui_ctx, |ui| {
        ui.label("内容");
    });
```

### Area（枠なし、自由配置）

HUD やオーバーレイ向き。

```rust
egui::Area::new(egui::Id::new("score"))
    .fixed_pos(egui::pos2(100.0, 50.0))
    .show(egui_ctx, |ui| {
        ui.label("SCORE: 1000");
    });
```

### CentralPanel / SidePanel / TopBottomPanel

ウィンドウいっぱいに広がるパネル。

```rust
egui::CentralPanel::default().show(egui_ctx, |ui| {
    ui.heading("メインコンテンツ");
});
```

---

## ウィジェット

### テキスト系

```rust
ui.label("普通のテキスト");
ui.heading("見出し");
ui.monospace("等幅フォント");

// リッチテキスト（色・サイズ・装飾）
ui.label(
    egui::RichText::new("赤い太字")
        .color(egui::Color32::RED)
        .size(20.0)
        .strong()
        .italics(),
);

// ハイパーリンク（クリックで外部ブラウザを開く）
ui.hyperlink("https://example.com");
ui.hyperlink_to("リンクテキスト", "https://example.com");
```

### ボタン

```rust
if ui.button("クリック").clicked() {
    println!("クリックされた");
}

// 装飾付き
let btn = egui::Button::new("特別なボタン")
    .fill(egui::Color32::DARK_BLUE);
if ui.add(btn).clicked() {
    // ...
}
```

### スライダー / 数値入力

```rust
let mut value: f32 = 50.0;
ui.add(egui::Slider::new(&mut value, 0.0..=100.0).text("音量"));

let mut count: usize = 4;
ui.add(egui::Slider::new(&mut count, 3..=5).text("色数"));

// 数値入力
ui.add(egui::DragValue::new(&mut value).speed(1.0));
```

### チェックボックス / ラジオ

```rust
let mut enabled = false;
ui.checkbox(&mut enabled, "有効化");

#[derive(PartialEq)]
enum Difficulty { Easy, Normal, Hard }
let mut diff = Difficulty::Normal;
ui.radio_value(&mut diff, Difficulty::Easy, "Easy");
ui.radio_value(&mut diff, Difficulty::Normal, "Normal");
ui.radio_value(&mut diff, Difficulty::Hard, "Hard");
```

### テキスト入力

```rust
let mut name = String::new();
ui.text_edit_singleline(&mut name);

let mut multiline = String::new();
ui.text_edit_multiline(&mut multiline);
```

### コンボボックス（ドロップダウン）

```rust
let mut selected = "A".to_string();
egui::ComboBox::from_label("選択")
    .selected_text(&selected)
    .show_ui(ui, |ui| {
        ui.selectable_value(&mut selected, "A".to_string(), "A");
        ui.selectable_value(&mut selected, "B".to_string(), "B");
    });
```

---

## レイアウト

```rust
// 横並び
ui.horizontal(|ui| {
    ui.label("名前:");
    ui.text_edit_singleline(&mut name);
    if ui.button("送信").clicked() { /* */ }
});

// 縦並び（デフォルト）
ui.vertical(|ui| {
    ui.label("行1");
    ui.label("行2");
});

// 区切り線
ui.separator();

// 余白
ui.add_space(20.0);

// 折りたたみ
ui.collapsing("詳細を表示", |ui| {
    ui.label("隠れた内容");
});

// グリッド
egui::Grid::new("my_grid").show(ui, |ui| {
    ui.label("名前");
    ui.text_edit_singleline(&mut name);
    ui.end_row();
    ui.label("年齢");
    ui.text_edit_singleline(&mut age);
    ui.end_row();
});
```

---

## スタイル変更

### スコープ内だけ一時変更

```rust
ui.scope(|ui| {
    ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
    ui.label("赤いテキスト");
});
ui.label("元の色");
```

### グローバル変更

```rust
egui_macroquad::ui(|ctx| {
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(20.0),
    );
    ctx.set_style(style);
});
```

### 透明ウィンドウ

```rust
egui::Window::new("透明")
    .frame(egui::Frame::NONE)
    .show(ctx, |ui| { /* */ });
```

---

## 入力の競合対策

egui がキー入力を使っている時はゲーム側のキー処理を止める:

```rust
egui_macroquad::ui(|egui_ctx| {
    if egui_ctx.wants_keyboard_input() {
        // テキスト入力中など → ゲームのキー入力をスキップ
    }
    if egui_ctx.wants_pointer_input() {
        // egui がマウスを使用中
    }
});
```

---

## このプロジェクトでの使用例

### 1. スコア表示（最初に検討した方式、現在は macroquad の text に変更済み）

```rust
pub fn draw_score(&self, score: u32) {
    egui_macroquad::ui(|ctx| {
        egui::Area::new(egui::Id::new("score"))
            .fixed_pos(egui::pos2(x, y))
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(format!("{score:08}"))
                        .size(28.0)
                        .strong()
                        .color(egui::Color32::from_rgb(255, 255, 100)),
                );
            });
    });
    egui_macroquad::draw();
}
```

### 2. 設定画面（現在採用中の `draw_settings`）

```rust
pub fn draw_settings(
    &self,
    puyo_colors: &mut usize,
    bgm_volume: &mut f32,
    showing_credits: &mut bool,
) -> bool {
    let mut close = false;
    let credits = *showing_credits;

    egui_macroquad::ui(|ctx| {
        egui::Window::new(if credits { "Credits" } else { "Settings" })
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                if credits {
                    ui.label("BGM (Niconico Commons):");
                    ui.hyperlink("https://commons.nicovideo.jp/works/agreement/nc2971");
                    ui.add_space(10.0);
                    if ui.button("Back").clicked() {
                        *showing_credits = false;
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Puyo colors:");
                        ui.add(egui::Slider::new(puyo_colors, 3..=5));
                    });
                    ui.horizontal(|ui| {
                        ui.label("BGM volume:");
                        ui.add(egui::Slider::new(bgm_volume, 0.0..=1.0));
                    });
                    ui.add_space(10.0);
                    if ui.button("Credits").clicked() {
                        *showing_credits = true;
                    }
                    if ui.button("Close (ESC)").clicked() {
                        close = true;
                    }
                }
            });
    });
    egui_macroquad::draw();
    close
}
```

---

## よく使う色定数

```rust
egui::Color32::WHITE
egui::Color32::BLACK
egui::Color32::RED
egui::Color32::GREEN
egui::Color32::BLUE
egui::Color32::YELLOW
egui::Color32::TRANSPARENT
egui::Color32::from_rgb(255, 100, 50)
egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128)  // 半透明
```

---

## ハマりどころ

### `egui_macroquad::draw()` を呼び忘れる
→ UI が表示されない。`ui()` の後に必ず呼ぶ。

### マクロクラックの描画と GL ステートの干渉
egui の描画は OpenGL ステートを変更する。GL を直接操作する処理（ステンシル等）と組み合わせる時は描画順に注意。基本は egui を最後に描く。

### キー入力がゲームに伝わってしまう
テキスト入力フィールドにフォーカスがあるのにゲーム側もキーを処理してしまう。`wants_keyboard_input()` でガードする。

### Immediate Mode の罠
状態は外部に保持する必要がある。クロージャ内で `let mut` しても次フレームには消える。

```rust
// NG
egui_macroquad::ui(|ctx| {
    let mut value = 0.0;  // 毎フレーム 0.0 にリセットされる
    ui.add(egui::Slider::new(&mut value, 0.0..=1.0));
});

// OK
struct AppState { value: f32 }
let state = &mut self.state;
egui_macroquad::ui(|ctx| {
    ui.add(egui::Slider::new(&mut state.value, 0.0..=1.0));
});
```
