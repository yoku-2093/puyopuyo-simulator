// ウィンドウサイズ
pub const WINDOW_WIDTH: f32 = 900.0;
pub const WINDOW_HEIGHT: f32 = 900.0;

// ぷよ1個あたりの描画サイズ（ピクセル）
pub const PUYO_SIZE: f32 = 60.0;

// 見えているフィールドの列数・行数
pub const COLS: usize = 6;
pub const ROWS: usize = 12;

// 落下・移動の時間間隔（秒）
pub const DROP_INTERVAL: f64 = 0.3;
pub const MOVE_INTERVAL: f64 = 0.05;
pub const MOVE_REPEAT_DELAY: f64 = 0.2; // 初回入力からリピート開始までの猶予
pub const LOCK_DELAY: f64 = 0.25; // 接地から固定までの猶予
pub const QUICK_TURN_WINDOW: f64 = 0.3;
pub const DROP_GRAVITY: f64 = 50.0; // ちぎり時の重力加速度（rows/s^2）
pub const DROP_GRAVITY_INITIAL: f64 = 5.0; // ちぎり開始時の初速（rows/s）
pub const DISPLAY_CHASE_RATE: f64 = 40.0; // 移動の表示位置追従速度
pub const ROTATION_CHASE_RATE: f64 = 20.0; // 回転の表示角度追従速度
pub const SQUASHING_ANIM_DURATION: f64 = 0.15; // 着地スカッシュの長さ
pub const SQUASHING_SQUASH_RATIO: f32 = 0.3; // 縦の潰れの最大量（0.3 = 30%縮む）
pub const BLINK_DURATION: f64 = 0.5; // 点滅の長さ
pub const SPARKLE_DURATION: f64 = 0.5; // キラキラの長さ
pub const SPARKLE_WAIT: f64 = 0.15; // パーティクル開始から落下開始までの待ち
pub const BLINK_COUNT: u32 = 5; // 点滅回数
pub const PARTICLE_COUNT: usize = 4; // 1ぷよあたりのパーティクル数
pub const PARTICLE_SPEED_MIN: f32 = 3.0; // パーティクルの最低速度（グリッド/秒）
pub const PARTICLE_SPEED_MAX: f32 = 8.0; // パーティクルの最高速度
pub const PARTICLE_GRAVITY: f32 = 15.0; // パーティクルの重力（グリッド/秒²）
pub const PARTICLE_SIZE_MIN: f32 = 0.08; // パーティクルの最小サイズ（グリッド単位）
pub const PARTICLE_SIZE_MAX: f32 = 0.18; // パーティクルの最大サイズ

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
