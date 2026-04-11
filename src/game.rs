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

    pub fn col(&self) -> usize {
        self.col
    }
    pub fn row(&self) -> usize {
        self.row
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

pub struct GameField {
    puyo: PuyoPuyo,                            // 落下中のぷよ
    position: Position,                        // 軸ぷよの位置
    next: PuyoPuyo,                            // 次のぷよ
    next_next: PuyoPuyo,                       // 次の次のぷよ
    field: [[Option<Puyo>; COLS]; TOTAL_ROWS], // フィールド（幽霊行を含む）
}

impl GameField {
    pub fn new() -> Self {
        GameField {
            puyo: PuyoPuyo::new(),
            position: INITIAL_POSITION,
            next: PuyoPuyo::new(),
            next_next: PuyoPuyo::new(),
            field: [[None; COLS]; TOTAL_ROWS],
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

    /// ちぎり落下（1マスずつ）。落ちきったら true
    fn drop_tick(&mut self) -> bool {
        !self.drop_down()
    }

    /// 幽霊行を除いた見える部分のフィールドを返す
    pub fn field(&self) -> &[[Option<Puyo>; COLS]] {
        &self.field[GHOST_ROWS..]
    }

    /// 落下中のぷよを返す（幽霊行のぷよは含まない）
    pub fn active(&self) -> Vec<(Puyo, Position)> {
        let puyos = [
            (self.puyo.axis(), self.position),
            (self.puyo.child(), self.child_position()),
        ];
        puyos
            .into_iter()
            .filter(|(_, pos)| pos.row >= GHOST_ROWS)
            .map(|(puyo, pos)| (puyo, Position::new(pos.col, pos.row - GHOST_ROWS)))
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

    // フィールドのぷよを1マスだけ落とす。落としたら true
    fn drop_down(&mut self) -> bool {
        let mut dropped = false;
        for row in (0..TOTAL_ROWS - 1).rev() {
            for col in 0..COLS {
                if self.field[row][col].is_some() && self.field[row + 1][col].is_none() {
                    self.field[row + 1][col] = self.field[row][col];
                    self.field[row][col] = None;
                    dropped = true;
                }
            }
        }
        dropped
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

    fn settle(&mut self) {
        // フィールドに固定
        let axis_pos = self.position;
        let child_pos = self.child_position();
        self.field[axis_pos.row][axis_pos.col] = Some(self.puyo.axis());
        self.field[child_pos.row][child_pos.col] = Some(self.puyo.child());
    }

    // ネクストぷよに切り替え
    fn spawn_next(&mut self) {
        self.puyo = self.next;
        self.next = self.next_next;
        self.next_next = PuyoPuyo::new();
        self.position = INITIAL_POSITION;
    }

    fn is_game_over(&self) -> bool {
        self.field[INITIAL_POSITION.row][INITIAL_POSITION.col].is_some()
    }
}

pub struct PlayContext {
    pub play_state: PlayState,
    last_drop_time: f64,
    last_move_time: f64,
    move_repeating: bool, // リピート移動が開始されているか
    settling_start: f64,  // 接地待ち開始時刻
    last_failed_rotation: Option<(Rotation, f64)>, // クイックターン判定用
}

impl PlayContext {
    pub fn new() -> Self {
        PlayContext {
            play_state: PlayState::Active,
            last_drop_time: get_time(),
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
        match ctx.play_state {
            PlayState::Active => {
                self.tick_active(ctx, now);
                false
            }
            PlayState::Settling => {
                self.tick_settling(ctx, now);
                false
            }
            PlayState::Dropping => {
                self.tick_dropping(ctx, now);
                false
            }
            PlayState::Landed => self.tick_landed(ctx, now),
        }
    }

    fn tick_active(&mut self, ctx: &mut PlayContext, now: f64) {
        if now - ctx.last_drop_time > DROP_INTERVAL {
            self.move_down();
            ctx.last_drop_time = now;
        }
        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx);
        if self.is_grounded() {
            ctx.play_state = PlayState::Settling;
            ctx.settling_start = now;
        }
    }

    fn tick_settling(&mut self, ctx: &mut PlayContext, now: f64) {
        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx);
        if !self.is_grounded() {
            ctx.play_state = PlayState::Active;
        } else if now - ctx.settling_start > LOCK_DELAY {
            self.settle();
            ctx.play_state = PlayState::Dropping;
        }
    }

    fn tick_dropping(&mut self, ctx: &mut PlayContext, now: f64) {
        if now - ctx.last_drop_time > DROP_GRAVITY_INTERVAL {
            if self.drop_tick() {
                ctx.play_state = PlayState::Landed;
            }
            ctx.last_drop_time = now;
        }
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
