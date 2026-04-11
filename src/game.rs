use crate::constants::*;
use crate::puyo::*;
use macroquad::prelude::*;

const GHOST_ROWS: usize = 1;
const INITIAL_POSITION: Position = Position::new(2, GHOST_ROWS);
const TOTAL_ROWS: usize = ROWS + GHOST_ROWS;

#[derive(Clone, Copy)]
pub struct Position {
    col: usize,
    row: usize,
}

impl Position {
    const fn new(col: usize, row: usize) -> Self {
        assert!(col < COLS, "col out of range");
        assert!(row < TOTAL_ROWS, "row out of range");
        Position { col, row }
    }
}

pub enum Screen {
    Title,              // タイトル画面
    Playing(GameField), // プレイ中
    GameOver,           // ゲームオーバー
}

impl Screen {
    pub fn new() -> Self {
        Screen::Title
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlayState {
    Active,   // 操作中
    Settling, // 接地して固定待ち
    Dropping, // ちぎり後の自由落下
    Landed,   // 接地完了
}

pub struct FloatingPuyo {
    pub puyo: Puyo,
    pub col: usize,
    pub row: f64,        // 現在の表示位置
    target_row: usize,   // 着地する行
    velocity: f64,       // 落下速度（rows/s）
}

pub struct GameField {
    puyo: PuyoPuyo,                            // 落下中のぷよ
    position: Position,                        // 軸ぷよの位置
    display_col: f64,                          // 軸の表示位置（補間用）
    display_row: f64,
    next: PuyoPuyo,                            // 次のぷよ
    next_next: PuyoPuyo,                       // 次の次のぷよ
    field: [[Option<Puyo>; COLS]; TOTAL_ROWS], // フィールド（幽霊行を含む）
    floating: Vec<FloatingPuyo>,               // ちぎり中のぷよ
}

impl GameField {
    pub fn new() -> Self {
        GameField {
            puyo: PuyoPuyo::new(),
            position: INITIAL_POSITION,
            display_col: INITIAL_POSITION.col as f64,
            display_row: INITIAL_POSITION.row as f64,
            next: PuyoPuyo::new(),
            next_next: PuyoPuyo::new(),
            field: [[None; COLS]; TOTAL_ROWS],
            floating: Vec::new(),
        }
    }

    fn child_position(&self) -> Position {
        let (dc, dr) = self.puyo.orientation().offset();
        Position::new(
            (self.position.col as isize + dc) as usize,
            (self.position.row as isize + dr) as usize,
        )
    }

    /// 接地しているか（下に動かせないか）
    fn is_grounded(&self) -> bool {
        !self.can_move(0, 1)
    }

    /// 幽霊行を除いた見える部分のフィールドを返す
    pub fn field(&self) -> &[[Option<Puyo>; COLS]] {
        &self.field[GHOST_ROWS..]
    }

    /// 操作中のぷよの表示位置（軸と子）。幽霊行は除外。
    pub fn active(&self) -> Vec<(Puyo, f32, f32)> {
        let (dc, dr) = self.puyo.orientation().offset();
        let axis = (
            self.puyo.axis(),
            self.display_col,
            self.display_row,
        );
        let child = (
            self.puyo.child(),
            self.display_col + dc as f64,
            self.display_row + dr as f64,
        );
        [axis, child]
            .into_iter()
            .filter(|(_, _, row)| *row >= (GHOST_ROWS as f64) - 0.5)
            .map(|(puyo, col, row)| (puyo, col as f32, (row - GHOST_ROWS as f64) as f32))
            .collect()
    }

    /// ちぎり中のぷよの表示位置
    pub fn floating(&self) -> Vec<(Puyo, f32, f32)> {
        self.floating
            .iter()
            .filter(|f| f.row >= (GHOST_ROWS as f64) - 0.5)
            .map(|f| (f.puyo, f.col as f32, (f.row - GHOST_ROWS as f64) as f32))
            .collect()
    }

    /// 移動後の軸と子の両方が範囲内かつ空きマスか判定
    fn can_move(&self, dc: isize, dr: isize) -> bool {
        let (child_dc, child_dr) = self.puyo.orientation().offset();
        let new_col = self.position.col as isize + dc;
        let new_row = self.position.row as isize + dr;
        let new_child_col = new_col + child_dc;
        let new_child_row = new_row + child_dr;
        self.is_empty(new_col, new_row) && self.is_empty(new_child_col, new_child_row)
    }

    /// 指定座標が範囲内かつ空きマスか判定
    fn is_empty(&self, col: isize, row: isize) -> bool {
        col >= 0
            && col < COLS as isize
            && row >= 0
            && row < TOTAL_ROWS as isize
            && self.field[row as usize][col as usize].is_none()
    }

    fn move_left(&mut self) {
        if self.can_move(-1, 0) {
            self.position.col -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.can_move(1, 0) {
            self.position.col += 1;
        }
    }

    fn move_down(&mut self) -> bool {
        if self.can_move(0, 1) {
            self.position.row += 1;
            true
        } else {
            false
        }
    }

    fn rotate(&mut self, rotation: Rotation, is_quick: bool) -> bool {
        let target_ori = if is_quick {
            self.puyo.orientation().rotate(rotation).rotate(rotation)
        } else {
            self.puyo.orientation().rotate(rotation)
        };
        let (dc, dr) = target_ori.offset();
        let col = self.position.col as isize;
        let row = self.position.row as isize;
        let cc = col + dc;
        let cr = row + dr;
        let kc = col - dc;
        let kr = row - dr;

        // 子ぷよの先もキック先も埋まっている → 失敗
        if !self.is_empty(cc, cr) && !self.is_empty(kc, kr) {
            return false;
        }

        // キックが必要なら軸をずらす
        if !self.is_empty(cc, cr) {
            self.position.col = kc as usize;
            self.position.row = kr as usize;
        }
        self.puyo.set_orientation(target_ori);
        true
    }

    // ネクストぷよに切り替え
    fn spawn_next(&mut self) {
        self.puyo = self.next;
        self.next = self.next_next;
        self.next_next = PuyoPuyo::new();
        self.position = INITIAL_POSITION;
        self.display_col = INITIAL_POSITION.col as f64;
        self.display_row = INITIAL_POSITION.row as f64;
    }

    fn is_game_over(&self) -> bool {
        self.field[INITIAL_POSITION.row][INITIAL_POSITION.col].is_some()
    }
}

pub struct PlayContext {
    pub play_state: PlayState,
    last_drop_time: f64,
    last_frame_time: f64, // 直前の tick の時刻（dt 計算用）
    last_move_time: f64,
    move_repeating: bool, // リピート移動が開始されているか
    settling_start: f64,  // 接地待ち開始時刻
    last_failed_rotation: Option<(Rotation, f64)>, // クイックターン判定用
}

impl PlayContext {
    pub fn new() -> Self {
        let now = get_time();
        PlayContext {
            play_state: PlayState::Active,
            last_drop_time: now,
            last_frame_time: now,
            last_move_time: 0.0,
            move_repeating: false,
            settling_start: 0.0,
            last_failed_rotation: None,
        }
    }
}

impl GameField {
    /// 戻り値: ゲームオーバーなら true
    pub fn tick(&mut self, ctx: &mut PlayContext, now: f64) -> bool {
        let dt = (now - ctx.last_frame_time).clamp(0.0, 0.1);
        ctx.last_frame_time = now;
        match ctx.play_state {
            PlayState::Active => {
                self.tick_active(ctx, now, dt);
                false
            }
            PlayState::Settling => {
                self.tick_settling(ctx, now, dt);
                false
            }
            PlayState::Dropping => {
                self.tick_dropping(ctx, dt);
                false
            }
            PlayState::Landed => self.tick_landed(ctx, now),
        }
    }

    fn tick_active(&mut self, ctx: &mut PlayContext, now: f64, dt: f64) {
        if now - ctx.last_drop_time > DROP_INTERVAL {
            self.move_down();
            ctx.last_drop_time = now;
        }
        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx);
        self.update_display(dt);
        if self.is_grounded() {
            ctx.play_state = PlayState::Settling;
            ctx.settling_start = now;
        }
    }

    fn tick_settling(&mut self, ctx: &mut PlayContext, now: f64, dt: f64) {
        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx);
        self.update_display(dt);
        if !self.is_grounded() {
            ctx.play_state = PlayState::Active;
        } else if now - ctx.settling_start > LOCK_DELAY
            && (self.display_row - self.position.row as f64).abs() < 0.05
        {
            self.start_chigiri();
            ctx.play_state = PlayState::Dropping;
        }
    }

    fn tick_dropping(&mut self, ctx: &mut PlayContext, dt: f64) {
        let mut i = 0;
        while i < self.floating.len() {
            let f = &mut self.floating[i];
            f.velocity += DROP_GRAVITY * dt;
            f.row += f.velocity * dt;
            if f.row >= f.target_row as f64 {
                let f = self.floating.remove(i);
                self.field[f.target_row][f.col] = Some(f.puyo);
            } else {
                i += 1;
            }
        }
        if self.floating.is_empty() {
            ctx.play_state = PlayState::Landed;
        }
    }

    /// 表示位置を論理位置に向かって補間
    fn update_display(&mut self, dt: f64) {
        let factor = (ACTIVE_FALL_LERP * dt).min(1.0);
        self.display_col += (self.position.col as f64 - self.display_col) * factor;
        self.display_row += (self.position.row as f64 - self.display_row) * factor;
    }

    /// 組ぷよを floating に積んでちぎりを開始
    fn start_chigiri(&mut self) {
        let axis_col = self.position.col;
        let axis_row = self.position.row;
        let child = self.child_position();
        let axis_puyo = self.puyo.axis();
        let child_puyo = self.puyo.child();

        if axis_col == child.col {
            // 同じ列: 下のぷよから順に target を割り当てる
            let col = axis_col;
            let mut bottom = self.bottom_empty(col);
            let (lower_puyo, lower_row, upper_puyo, upper_row) = if axis_row > child.row {
                (axis_puyo, axis_row, child_puyo, child.row)
            } else {
                (child_puyo, child.row, axis_puyo, axis_row)
            };
            self.floating.push(FloatingPuyo {
                puyo: lower_puyo,
                col,
                row: lower_row as f64,
                target_row: bottom,
                velocity: DROP_GRAVITY_INITIAL,
            });
            bottom = bottom.saturating_sub(1);
            self.floating.push(FloatingPuyo {
                puyo: upper_puyo,
                col,
                row: upper_row as f64,
                target_row: bottom,
                velocity: DROP_GRAVITY_INITIAL,
            });
        } else {
            // 別の列: 独立に target を計算
            let axis_target = self.bottom_empty(axis_col);
            let child_target = self.bottom_empty(child.col);
            self.floating.push(FloatingPuyo {
                puyo: axis_puyo,
                col: axis_col,
                row: axis_row as f64,
                target_row: axis_target,
                velocity: DROP_GRAVITY_INITIAL,
            });
            self.floating.push(FloatingPuyo {
                puyo: child_puyo,
                col: child.col,
                row: child.row as f64,
                target_row: child_target,
                velocity: DROP_GRAVITY_INITIAL,
            });
        }
    }

    fn bottom_empty(&self, col: usize) -> usize {
        for r in (0..TOTAL_ROWS).rev() {
            if self.field[r][col].is_none() {
                return r;
            }
        }
        0
    }

    fn tick_landed(&mut self, ctx: &mut PlayContext, now: f64) -> bool {
        if self.is_game_over() {
            return true;
        }
        self.spawn_next();
        ctx.play_state = PlayState::Active;
        ctx.last_drop_time = now;
        false
    }

    fn handle_move_keys(&mut self, ctx: &mut PlayContext, now: f64) {
        let dirs = [KeyCode::Left, KeyCode::Right, KeyCode::Down];
        let just_pressed = dirs.iter().any(|&k| is_key_pressed(k));
        let held = dirs.iter().any(|&k| is_key_down(k));

        if !held {
            ctx.move_repeating = false;
            return;
        }

        let interval = if ctx.move_repeating {
            MOVE_INTERVAL
        } else {
            MOVE_REPEAT_DELAY
        };
        if !just_pressed && now - ctx.last_move_time <= interval {
            return;
        }

        if is_key_down(KeyCode::Left) {
            self.move_left();
        }
        if is_key_down(KeyCode::Right) {
            self.move_right();
        }
        if is_key_down(KeyCode::Down) {
            self.move_down();
        }
        ctx.last_move_time = now;
        ctx.move_repeating = !just_pressed;
    }

    fn handle_rotate_keys(&mut self, ctx: &mut PlayContext) {
        if is_key_pressed(KeyCode::X) {
            self.try_rotate(ctx, Rotation::Right);
        }
        if is_key_pressed(KeyCode::Z) {
            self.try_rotate(ctx, Rotation::Left);
        }
    }

    fn try_rotate(&mut self, ctx: &mut PlayContext, rotation: Rotation) {
        let now = get_time();
        let is_quick = matches!(
            ctx.last_failed_rotation,
            Some((r, t)) if r == rotation && now - t < QUICK_TURN_WINDOW
        );

        if self.rotate(rotation, is_quick) {
            ctx.last_failed_rotation = None;
        } else {
            ctx.last_failed_rotation = Some((rotation, now));
        }
    }
}
