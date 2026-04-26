// 見えているフィールドの列数・行数
pub const COLS: usize = 6;
pub const ROWS: usize = 12;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Puyo {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}
