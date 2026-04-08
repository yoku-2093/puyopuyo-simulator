// ウィンドウサイズ
pub const WINDOW_WIDTH: f32 = 900.0;
pub const WINDOW_HEIGHT: f32 = 900.0;

// ぷよ1個あたりの描画サイズ（ピクセル）
pub const PUYO_SIZE: f32 = 60.0;

// フィールドの列数・行数
pub const COLS: usize = 6;
pub const ROWS: usize = 12;

// 落下・移動の時間間隔（秒）
pub const DROP_INTERVAL: f64 = 0.4;
pub const MOVE_INTERVAL: f64 = 0.1;
pub const QUICK_TURN_WINDOW: f64 = 0.3;

// フィールド左上のピクセル座標（ウィンドウ中央に配置）
pub const FIELD_X: f32 = (WINDOW_WIDTH - PUYO_SIZE * COLS as f32) / 2.0;
pub const FIELD_Y: f32 = (WINDOW_HEIGHT - PUYO_SIZE * ROWS as f32) / 2.0;
