# macroquad モジュールガイド

## shapes（2D 図形描画）

図形を描画するための関数群。

| 関数 | 用途 |
|---|---|
| `draw_circle(x, y, r, color)` | 塗りつぶし円 |
| `draw_circle_lines(x, y, r, thickness, color)` | 円のアウトライン |
| `draw_rectangle(x, y, w, h, color)` | 塗りつぶし矩形 |
| `draw_rectangle_lines(x, y, w, h, thickness, color)` | 矩形のアウトライン |
| `draw_line(x1, y1, x2, y2, thickness, color)` | 直線 |
| `draw_triangle(v1, v2, v3, color)` | 三角形 |
| `draw_ellipse(x, y, w, h, rotation, color)` | 楕円 |
| `draw_arc(x, y, sides, radius, rotation, arc, thickness, color)` | 弧 |
| `draw_poly(x, y, sides, radius, rotation, color)` | 正多角形 |
| `draw_hexagon(x, y, size, border, vertical, border_color, fill_color)` | 六角形 |

```rust
// ぷよを1個描画
draw_circle(100.0, 200.0, 16.0, RED);                   // 塗りつぶし
draw_circle_lines(100.0, 200.0, 16.0, 2.0, DARKGRAY);   // 枠線
```

## input（入力処理）

### キーボード

| 関数 | 用途 |
|---|---|
| `is_key_pressed(KeyCode)` | そのフレームで押された瞬間を検出（移動・回転に最適） |
| `is_key_down(KeyCode)` | 押し続けている間 true（高速落下に最適） |
| `is_key_released(KeyCode)` | 離した瞬間を検出 |
| `get_last_key_pressed()` | 最後に押されたキーを返す |
| `get_char_pressed()` | 文字入力を取得 |

### マウス

| 関数 | 用途 |
|---|---|
| `mouse_position()` | マウス位置（ピクセル） |
| `is_mouse_button_pressed(MouseButton)` | クリック検出 |
| `mouse_wheel()` | マウスホイール |

### KeyCode の例

`KeyCode::Left`, `KeyCode::Right`, `KeyCode::Down`, `KeyCode::Up`, `KeyCode::Space`, `KeyCode::Z`, `KeyCode::X` など

```rust
if is_key_pressed(KeyCode::Left) {
    // ぷよを左に移動
}
if is_key_pressed(KeyCode::Right) {
    // ぷよを右に移動
}
if is_key_pressed(KeyCode::Z) {
    // 左回転
}
if is_key_pressed(KeyCode::X) {
    // 右回転
}
if is_key_down(KeyCode::Down) {
    // 高速落下
}
```

### 注意: 描画と状態の分離

`is_key_pressed` は押した瞬間の1フレームだけ `true` を返す。描画関数も1フレームだけ有効。
そのため、入力に応じて**状態を更新**し、毎フレームその状態に基づいて描画する設計が必要。

```rust
let mut show_rect = false;

loop {
    clear_background(WHITE);

    if is_key_pressed(KeyCode::Space) {
        show_rect = !show_rect; // 状態を更新
    }

    if show_rect {
        draw_rectangle(300.0, 300.0, 100.0, 50.0, RED); // 状態に基づいて描画
    }

    next_frame().await;
}
```

## texture（テクスチャ・画像）

### 主な型

| 型 | 説明 |
|---|---|
| `Texture2D` | GPU メモリ上のテクスチャ |
| `Image` | CPU メモリ上の画像データ |
| `DrawTextureParams` | 描画オプション（ソース矩形、回転など） |

### 主な関数

| 関数 | 用途 |
|---|---|
| `load_texture("path.png").await` | 画像ファイルを GPU テクスチャとして読み込み |
| `load_image("path.png").await` | 画像を CPU メモリに読み込み |
| `draw_texture(texture, x, y, color)` | テクスチャを描画 |
| `draw_texture_ex(texture, x, y, color, params)` | 詳細パラメータ付きで描画 |
| `set_default_filter_mode(FilterMode)` | フィルタモード設定（ドット絵なら `Nearest`） |
| `get_screen_data()` | スクリーンショットを取得 |

```rust
// 起動時にテクスチャ読み込み
let puyo_texture = load_texture("assets/puyo.png").await.unwrap();

// ドット絵風にするならフィルタを Nearest に
puyo_texture.set_filter(FilterMode::Nearest);

// スプライトシートから一部を切り出して描画
draw_texture_ex(
    &puyo_texture,
    x, y,
    WHITE,
    DrawTextureParams {
        source: Some(Rect::new(0.0, 0.0, 32.0, 32.0)),
        dest_size: Some(Vec2::new(32.0, 32.0)),
        ..Default::default()
    },
);
```

## text（テキスト描画）

| 関数 | 用途 |
|---|---|
| `draw_text(text, x, y, font_size, color)` | テキストを描画 |
| `draw_text_ex(text, x, y, params)` | 詳細パラメータ付きで描画 |
| `measure_text(text, font, font_size, font_scale)` | テキストのピクセル幅・高さを取得 |
| `load_ttf_font("path.ttf").await` | カスタムフォントの読み込み |

```rust
// 基本
draw_text("SCORE: 1000", 20.0, 40.0, 30.0, WHITE);

// テキスト幅を測って右寄せ
let text = "Game Over";
let size = measure_text(text, None, 40, 1.0);
draw_text(text, screen_width() / 2.0 - size.width / 2.0, 300.0, 40.0, RED);
```

## time（時間・FPS）

| 関数 | 用途 |
|---|---|
| `get_time()` | アプリ起動からの経過秒数（f64） |
| `get_fps()` | 現在の FPS |
| `get_frame_time()` | 前フレームからの経過秒数（f32） |

```rust
// FPS 表示
draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, WHITE);

// フレームレートに依存しない移動
let speed = 200.0; // ピクセル/秒
x += speed * get_frame_time();

// 一定間隔で処理（例: 0.5秒ごとにぷよ落下）
let mut last_drop = 0.0;
// loop 内で:
if get_time() - last_drop > 0.5 {
    // ぷよを1段落とす
    last_drop = get_time();
}
```

## window（ウィンドウ）

| 関数 | 用途 |
|---|---|
| `screen_width()` | ウィンドウ幅 |
| `screen_height()` | ウィンドウ高さ |
| `request_new_screen_size(w, h)` | ウィンドウサイズを変更 |
| `next_frame().await` | フレームを進める（ループ末尾で必須） |

```rust
// ウィンドウ中央の座標
let cx = screen_width() / 2.0;
let cy = screen_height() / 2.0;
```

## camera（カメラ）

2D ゲームでもスクロールやズームに使える。

| 関数 | 用途 |
|---|---|
| `set_camera(&Camera2D { ... })` | カメラを設定 |
| `set_default_camera()` | デフォルトカメラに戻す |

```rust
// ズームイン（2倍に拡大）
set_camera(&Camera2D {
    zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
    target: vec2(200.0, 300.0), // カメラの注視点
    ..Default::default()
});

// 描画後にデフォルトに戻す（UI はカメラの影響を受けたくない場合）
set_default_camera();
```

## audio（サウンド）

| 関数 | 用途 |
|---|---|
| `load_sound("path.wav").await` | サウンドファイルを読み込み |
| `play_sound_once(&sound)` | 1回再生 |
| `play_sound(&sound, params)` | パラメータ付きで再生 |
| `stop_sound(&sound)` | 停止 |
| `set_sound_volume(&sound, volume)` | 音量設定（0.0〜1.0） |

```rust
let bgm = load_sound("assets/bgm.wav").await.unwrap();
let se_chain = load_sound("assets/chain.wav").await.unwrap();

// BGM をループ再生
play_sound(
    &bgm,
    PlaySoundParams { looped: true, volume: 0.5 },
);

// 連鎖時に SE を鳴らす
play_sound_once(&se_chain);
```

## color（色）

### 定義済み色
`RED`, `BLUE`, `GREEN`, `YELLOW`, `WHITE`, `BLACK`, `GRAY`, `LIGHTGRAY`, `DARKGRAY`, `ORANGE`, `PINK`, `PURPLE`, `MAGENTA` など

### カスタム色

```rust
// RGBA（0.0〜1.0）
let my_color = Color::new(0.2, 0.5, 1.0, 1.0);

// RGBA（0〜255）マクロ
let my_color = color_u8!(50, 128, 255, 255);

// 半透明
let transparent_red = Color::new(1.0, 0.0, 0.0, 0.5);
```

## rand（乱数）

macroquad 内蔵の乱数モジュール。

| 関数 | 用途 |
|---|---|
| `rand::gen_range(min, max)` | min〜max の乱数 |
| `rand::srand(seed)` | シードを設定（再現性が必要な場合） |
| `rand::rand()` | u32 の乱数 |

```rust
use macroquad::rand;

// ぷよの色をランダムに決定
let colors = [PuyoColor::Red, PuyoColor::Blue, PuyoColor::Green, PuyoColor::Yellow, PuyoColor::Purple];
let random_color = colors[rand::gen_range(0, colors.len())];

// シード固定（同じ手順で同じ結果を再現）
rand::srand(12345);
```

## conf（ウィンドウ初期設定）

`#[macroquad::main]` に渡す設定関数で使用。

```rust
fn window_conf() -> Conf {
    Conf {
        window_title: "PuyoPuyo Simulator".to_string(),
        window_width: 600,
        window_height: 900,
        window_resizable: false,
        high_dpi: false,       // Retina対応（trueだと座標が2倍になる）
        fullscreen: false,
        sample_count: 1,       // アンチエイリアス（1=なし）
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() { /* ... */ }
```

## math（幾何型・ベクトル）

`glam` クレートの再エクスポート。`use macroquad::prelude::*` で使える。

### よく使う型

| 型 | 説明 |
|---|---|
| `Vec2` | 2D ベクトル（x, y） |
| `IVec2` | 整数版 2D ベクトル |
| `Rect` | 矩形（x, y, w, h） |
| `Circle` | 円 |

### Vec2

```rust
// 生成
let v = vec2(100.0, 200.0);
let v = Vec2::new(100.0, 200.0);
let zero = Vec2::ZERO;

// 演算
let sum = v + vec2(10.0, 20.0);
let scaled = v * 2.0;
let len = v.length();
let dist = v.distance(other);
let normalized = v.normalize();
let lerped = v.lerp(other, 0.5); // 線形補間（アニメーションに便利）
```

### Rect

```rust
// 生成（左上座標 + 幅高さ）
let r = Rect::new(100.0, 200.0, 50.0, 30.0);

// 当たり判定
r.contains(vec2(120.0, 210.0));   // 点が矩形内か
r.overlaps(&other_rect);           // 矩形同士の重なり

// 位置・サイズ取得
r.center();  // 中心座標（Vec2）
r.left();    // x
r.right();   // x + w
r.top();     // y
r.bottom();  // y + h

// 移動
r.offset(vec2(10.0, 5.0));  // 移動した新しい Rect を返す
```

```rust
// クリック判定の典型パターン
let button_rect = Rect::new(100.0, 100.0, 200.0, 50.0);
draw_rectangle(button_rect.x, button_rect.y, button_rect.w, button_rect.h, BLUE);

if is_mouse_button_pressed(MouseButton::Left) {
    let (mx, my) = mouse_position();
    if button_rect.contains(vec2(mx, my)) {
        println!("ボタンが押された！");
    }
}
```

### 便利関数

| 関数 | 用途 |
|---|---|
| `clamp(value, min, max)` | 値を範囲内に収める |
| `vec2(x, y)` | Vec2 のショートハンド |

## ui（即時モード UI）

megaui ベースの組み込み UI。`root_ui()` がエントリーポイント。

### 基本ウィジェット

```rust
use macroquad::ui::root_ui;

// ボタン（押されたら true）
if root_ui().button(None, "スタート") {
    println!("押された！");
}

// ラベル
root_ui().label(None, "スコア: 1000");

// チェックボックス
let mut enabled = false;
root_ui().checkbox(hash!(), "BGM", &mut enabled);

// スライダー
let mut volume = 0.5;
root_ui().slider(hash!(), "音量", 0.0..1.0, &mut volume);

// テキスト入力
let mut name = String::new();
root_ui().input_text(hash!(), "名前", &mut name);

// コンボボックス
let items = ["Easy", "Normal", "Hard"];
let selected = root_ui().combo_box(hash!(), "難易度", &items, None);
```

`hash!()` はウィジェットの内部状態管理用 ID。同じフレーム内でユニークであればよい。

### ウィンドウ

```rust
use macroquad::ui::{root_ui, hash, widgets};

widgets::Window::new(hash!(), vec2(100.0, 50.0), vec2(300.0, 200.0))
    .label("設定")
    .movable(true)        // ドラッグ移動可能
    .close_button(true)   // 閉じるボタン
    .ui(&mut root_ui(), |ui| {
        if ui.button(None, "OK") {
            println!("OK");
        }
        ui.same_line(0.0);  // 横に並べる
        if ui.button(None, "Cancel") {
            println!("Cancel");
        }
    });
```

### 利用可能なウィジェット一覧

| ウィジェット | 用途 |
|---|---|
| `Button` | ボタン |
| `Label` | テキスト表示 |
| `Checkbox` | チェックボックス |
| `Slider` | スライダー |
| `InputText` | テキスト入力 |
| `Editbox` | 複数行テキスト入力 |
| `ComboBox` | ドロップダウン |
| `Tabbar` | タブ切り替え |
| `Group` | ウィジェットのグループ化 |
| `TreeNode` | ツリー表示 |
| `Texture` | 画像表示 |
| `Popup` | ポップアップ（最前面に描画） |
| `Window` | ウィンドウ |

## coroutines（コルーチン）

`macroquad::experimental::coroutines` にある、時間をまたぐ非同期処理の仕組み。
連鎖アニメーションや演出のように「待ちながら段階的に進む」処理に便利。

```rust
use macroquad::experimental::coroutines::{start_coroutine, wait_seconds, Coroutine};

// コルーチンを開始
let cor: Coroutine = start_coroutine(async move {
    // 1秒待つ
    wait_seconds(1.0).await;
    println!("1秒経過");

    // さらに0.5秒待つ
    wait_seconds(0.5).await;
    println!("さらに0.5秒経過");
});

// メインループ内でコルーチンの完了チェック
if cor.is_done() {
    println!("コルーチン完了");
}
```

```rust
// 連鎖アニメーションへの応用イメージ
start_coroutine(async move {
    for chain in 1..=chain_count {
        // ぷよを消すアニメーション
        wait_seconds(0.3).await;

        // 落下アニメーション
        wait_seconds(0.2).await;
    }
});
```

### 主な関数

| 関数 | 用途 |
|---|---|
| `start_coroutine(future)` | コルーチンを開始。`Coroutine` ハンドルを返す |
| `wait_seconds(secs)` | 指定秒数待機する Future |
| `stop_coroutine(cor)` | コルーチンを中断 |
| `stop_all_coroutines()` | 全コルーチンを停止 |

### 注意点
- コルーチン内から外部の状態を変更するには `Rc<RefCell<T>>` などの共有参照が必要（`move` で所有権を渡すため）
- `next_frame().await` をコルーチン内で呼ぶと1フレーム待機できる

## ゲーム状態管理パターン

macroquad に組み込みの状態管理はないが、enum + match が定番。

```rust
enum GameState {
    Title,
    Playing,
    Paused,
    GameOver,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::Title;

    loop {
        match state {
            GameState::Title => {
                draw_text("PUYO PUYO", 200.0, 300.0, 60.0, WHITE);
                draw_text("Press SPACE to start", 180.0, 400.0, 30.0, GRAY);
                if is_key_pressed(KeyCode::Space) {
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                // ゲームロジック & 描画
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Paused;
                }
            }
            GameState::Paused => {
                draw_text("PAUSED", 250.0, 300.0, 50.0, YELLOW);
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                draw_text("GAME OVER", 200.0, 300.0, 60.0, RED);
                if is_key_pressed(KeyCode::Space) {
                    state = GameState::Title;
                }
            }
        }

        next_frame().await;
    }
}
```

状態ごとにデータが異なる場合は enum にデータを持たせる：

```rust
enum GameState {
    Title,
    Playing { score: u32, field: Field },
    GameOver { final_score: u32 },
}
```

## 描画順序の注意点

macroquad は **後から描いたものが上に描画される**（ペインタアルゴリズム）。Z バッファはない。

```rust
// NG: テキストが背景で隠れる
draw_text("Hello", 100.0, 100.0, 30.0, WHITE);
draw_rectangle(0.0, 0.0, 800.0, 600.0, BLACK);  // テキストを覆い隠す

// OK: 背景 → フィールド → ぷよ → UI の順に描画
draw_background();    // 最背面
draw_field();         // フィールド枠
draw_puyos();         // ぷよ
draw_score();         // スコア表示（最前面）
```

`root_ui()` のウィジェットはフレーム最後に自動で描画されるため、常にゲーム描画の上に表示される。
