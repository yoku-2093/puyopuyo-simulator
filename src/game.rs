use crate::constants::*;
use crate::puyo::*;
use macroquad::prelude::*;

const GHOST_ROWS: usize = 1;
const TOTAL_ROWS: usize = ROWS + GHOST_ROWS;

#[derive(Clone, Copy)]
pub struct Position {
    col: usize,
    row: usize,
}

impl Position {
    fn new(col: usize, row: usize) -> Self {
        assert!(col < COLS, "col out of range: {}", col);
        assert!(
            row >= GHOST_ROWS && row < TOTAL_ROWS,
            "row out of range: {}",
            row
        );
        Position { col, row }
    }

    pub fn col(&self) -> usize {
        self.col
    }
    pub fn row(&self) -> usize {
        self.row
    }
}

pub struct Game {
    tsumo: KumiPuyo,
    position: Position,
    next_tsumo: KumiPuyo,
    next_next_tsumo: KumiPuyo,
    last_drop_time: f64,
    field: [[Option<PuyoColor>; COLS]; TOTAL_ROWS],
}

impl Game {
    pub fn new() -> Self {
        Game {
            tsumo: KumiPuyo::new(),
            position: Position::new(2, GHOST_ROWS),
            next_tsumo: KumiPuyo::new(),
            next_next_tsumo: KumiPuyo::new(),
            last_drop_time: get_time(),
            field: [[None; COLS]; TOTAL_ROWS],
        }
    }

    pub fn update(&mut self) {
        let now = get_time();

        // 0.5秒ごとに自動落下
        if now - self.last_drop_time > 0.5 {
            if !self.move_down() {
                self.ground(); // 着地 → 固定
            }
            self.last_drop_time = now;
        }

        // 方向キーの入力を処理
        if is_key_down(KeyCode::Left) {
            self.move_left();
        }
        if is_key_down(KeyCode::Right) {
            self.move_right();
        }
        if is_key_down(KeyCode::Down) {
            self.move_down();
        }
        if is_key_pressed(KeyCode::X) {
            self.rotate(Rotation::Right);
        }
        if is_key_pressed(KeyCode::Z) {
            self.rotate(Rotation::Left);
        }
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
            .map(|(color, pos)| {
                (
                    color,
                    Position {
                        col: pos.col,
                        row: pos.row - GHOST_ROWS,
                    },
                )
            })
            .collect()
    }

    /// 移動後の軸と子の両方が範囲内か判定
    fn can_move(&self, dc: isize, dr: isize) -> bool {
        let (child_dc, child_dr) = self.child_offset();
        let new_col = self.position.col as isize + dc;
        let new_row = self.position.row as isize + dr;
        let new_child_col = new_col + child_dc;
        let new_child_row = new_row + child_dr;
        new_col >= 0
            && new_col < COLS as isize
            && new_row >= 0
            && new_row < TOTAL_ROWS as isize
            && new_child_col >= 0
            && new_child_col < COLS as isize
            && new_child_row >= 0
            && new_child_row < TOTAL_ROWS as isize
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

    fn rotate(&mut self, rotation: Rotation) {
        self.tsumo.rotate(rotation);
        let (dc, dr) = self.child_offset();
        let child_col = self.position.col as isize + dc;
        let child_row = self.position.row as isize + dr;
        // 壁キック：子ぷよがはみ出したら軸ぷよを反対にずらす
        if child_col < 0 {
            self.position.col += 1;
        } else if child_col >= COLS as isize {
            self.position.col -= 1;
        }
        if child_row >= TOTAL_ROWS as isize {
            self.position.row -= 1;
        }
    }

    /// 軸ぷよに対する子ぷよの相対位置 (列差, 行差) を返す
    fn child_offset(&self) -> (isize, isize) {
        match self.tsumo.orientation() {
            Orientation::Up => (0, -1),
            Orientation::Right => (1, 0),
            Orientation::Down => (0, 1),
            Orientation::Left => (-1, 0),
        }
    }

    fn child_position(&self) -> Position {
        let (dc, dr) = self.child_offset();
        Position {
            col: (self.position.col as isize + dc) as usize,
            row: (self.position.row as isize + dr) as usize,
        }
    }

    fn ground(&mut self) {
        // フィールドに固定
        let axis_pos = self.position;
        let child_pos = self.child_position();
        self.field[axis_pos.row][axis_pos.col] = Some(self.tsumo.axis_color());
        self.field[child_pos.row][child_pos.col] = Some(self.tsumo.child_color());

        // 新しいツモを生成
        self.tsumo = self.next_tsumo;
        self.next_tsumo = self.next_next_tsumo;
        self.next_next_tsumo = KumiPuyo::new();
        self.position = Position::new(2, GHOST_ROWS);
    }
}
