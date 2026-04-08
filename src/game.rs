use crate::constants::*;
use crate::puyo::*;

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

pub struct GameField {
    tsumo: KumiPuyo,                                // 落下中の組ぷよ
    position: Position,                             // 軸ぷよの位置
    next_tsumo: KumiPuyo,                           // 次の組ぷよ
    next_next_tsumo: KumiPuyo,                      // 次の次の組ぷよ
    field: [[Option<PuyoColor>; COLS]; TOTAL_ROWS], // フィールド（幽霊行を含む）
    last_failed_rotation: Option<(Rotation, f64)>,  // クイックターン判定用
}

impl GameField {
    pub fn new() -> Self {
        GameField {
            tsumo: KumiPuyo::new(),
            position: INITIAL_POSITION,
            next_tsumo: KumiPuyo::new(),
            next_next_tsumo: KumiPuyo::new(),
            field: [[None; COLS]; TOTAL_ROWS],
            last_failed_rotation: None,
        }
    }

    /// 自動落下を実行
    pub fn tick(&mut self) -> TickResult {
        if !self.move_down() {
            self.settle();
            if self.is_game_over() {
                return TickResult::GameOver;
            }
            return TickResult::Settled;
        }
        TickResult::Falling
    }

    /// 幽霊行を除いた見える部分のフィールドを返す
    pub fn field(&self) -> &[[Option<PuyoColor>; COLS]] {
        &self.field[GHOST_ROWS..]
    }

    /// 落下中のぷよを返す（幽霊行のぷよは含まない）
    pub fn falling(&self) -> Vec<(PuyoColor, Position)> {
        let puyos = [
            (self.tsumo.axis_color(), self.position),
            (self.tsumo.child_color(), self.child_position()),
        ];
        puyos
            .into_iter()
            .filter(|(_, pos)| pos.row >= GHOST_ROWS)
            .map(|(color, pos)| (color, Position::new(pos.col, pos.row - GHOST_ROWS)))
            .collect()
    }

    /// 移動後の軸と子の両方が範囲内かつ空きマスか判定
    fn can_move(&self, dc: isize, dr: isize) -> bool {
        let (child_dc, child_dr) = self.tsumo.orientation().offset();
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

    pub fn move_left(&mut self) {
        if self.can_move(-1, 0) {
            self.position.col -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.can_move(1, 0) {
            self.position.col += 1;
        }
    }

    pub fn move_down(&mut self) -> bool {
        if self.can_move(0, 1) {
            self.position.row += 1;
            true
        } else {
            false
        }
    }

    pub fn rotate(&mut self, rotation: Rotation) {
        let now = macroquad::time::get_time();
        let is_quick = matches!(
            self.last_failed_rotation,
            Some((r, t)) if r == rotation && now - t < QUICK_TURN_WINDOW
        );

        let target_ori = if is_quick {
            self.tsumo.orientation().rotate(rotation).rotate(rotation)
        } else {
            self.tsumo.orientation().rotate(rotation)
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
            if !is_quick {
                self.last_failed_rotation = Some((rotation, now));
            }
            return;
        }

        // キックが必要なら軸をずらす
        if !self.is_empty(cc, cr) {
            self.position.col = kc as usize;
            self.position.row = kr as usize;
        }
        self.tsumo.set_orientation(target_ori);
        self.last_failed_rotation = None;
    }

    fn child_position(&self) -> Position {
        let (dc, dr) = self.tsumo.orientation().offset();
        Position::new(
            (self.position.col as isize + dc) as usize,
            (self.position.row as isize + dr) as usize,
        )
    }

    fn settle(&mut self) {
        // フィールドに固定
        let axis_pos = self.position;
        let child_pos = self.child_position();
        self.field[axis_pos.row][axis_pos.col] = Some(self.tsumo.axis_color());
        self.field[child_pos.row][child_pos.col] = Some(self.tsumo.child_color());

        // 新しいツモを生成
        self.tsumo = self.next_tsumo;
        self.next_tsumo = self.next_next_tsumo;
        self.next_next_tsumo = KumiPuyo::new();
        self.position = INITIAL_POSITION;
    }

    fn is_game_over(&self) -> bool {
        self.field[INITIAL_POSITION.row][INITIAL_POSITION.col].is_some()
    }
}

#[derive(PartialEq)]
pub enum TickResult {
    Falling,
    Settled,
    GameOver,
}

pub enum GamePhase {
    Start,
    Playing(GameField),
    GameOver(GameField),
}

impl GamePhase {
    pub fn new() -> Self {
        GamePhase::Start
    }
}
