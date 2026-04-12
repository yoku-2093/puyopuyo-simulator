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
pub const MOVE_REPEAT_DELAY: f64 = 0.2; // 初回入力からリピート開始までの猶予
pub const LOCK_DELAY: f64 = 0.25; // 接地から固定までの猶予
pub const QUICK_TURN_WINDOW: f64 = 0.3;
pub const DROP_GRAVITY: f64 = 50.0; // ちぎり時の重力加速度（rows/s^2）
pub const DROP_GRAVITY_INITIAL: f64 = 5.0; // ちぎり開始時の初速（rows/s）
pub const DISPLAY_CHASE_RATE: f64 = 40.0; // 操作中の表示位置追従係数
pub const LANDING_ANIM_DURATION: f64 = 0.15; // 着地スカッシュの長さ
pub const LANDING_SQUASH_RATIO: f32 = 0.3; // 縦の潰れの最大量（0.3 = 30%縮む）

// フィールド左上のピクセル座標（ウィンドウ中央に配置）
pub const FIELD_X: f32 = (WINDOW_WIDTH - PUYO_SIZE * COLS as f32) / 2.0;
pub const FIELD_Y: f32 = (WINDOW_HEIGHT - PUYO_SIZE * ROWS as f32) / 2.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Puyo {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}
