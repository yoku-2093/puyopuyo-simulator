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
        assert!(row >= GHOST_ROWS && row < TOTAL_ROWS, "row out of range: {}", row);
        Position { col, row }
    }

    pub fn col(&self) -> usize {
        self.col
    }
    pub fn row(&self) -> usize {
        self.row
    }
    fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.col + 1 < COLS {
            self.col += 1;
        }
    }

    fn move_down(&mut self) -> bool {
        if self.row + 1 < TOTAL_ROWS {
            self.row += 1;
            true
        } else {
            false // これ以上下がれない
        }
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
            if !self.position.move_down() {
                self.ground(); // 着地 → 固定
            }
            self.last_drop_time = now;
        }

        // 方向キーの入力を���理
        if is_key_down(KeyCode::Left) {
            self.position.move_left();
        }
        if is_key_down(KeyCode::Right) {
            self.position.move_right();
        }
        if is_key_down(KeyCode::Down) {
            self.position.move_down();
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
        puyos.into_iter()
            .filter(|(_, pos)| pos.row >= GHOST_ROWS)
            .map(|(color, pos)| (color, Position { col: pos.col, row: pos.row - GHOST_ROWS }))
            .collect()
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

    fn child_offset(&self) -> (isize, isize) {
        match self.tsumo.orientation() {
            Orientation::Up    => ( 0, -1),
            Orientation::Right => ( 1,  0),
            Orientation::Down  => ( 0,  1),
            Orientation::Left  => (-1,  0),
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
