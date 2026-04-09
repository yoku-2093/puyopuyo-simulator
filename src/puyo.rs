use macroquad::rand;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Puyo {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Up,    // 子が軸の上
    Right, // 子が軸の右
    Down,  // 子が軸の下
    Left,  // 子が軸の左
}

impl Orientation {
    const ALL: [Orientation; 4] = [
        Orientation::Up,
        Orientation::Right,
        Orientation::Down,
        Orientation::Left,
    ];

    /// 軸ぷよに対する子ぷよの相対位置 (列差, 行差)
    pub fn offset(self) -> (isize, isize) {
        match self {
            Orientation::Up => (0, -1),
            Orientation::Right => (1, 0),
            Orientation::Down => (0, 1),
            Orientation::Left => (-1, 0),
        }
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        let i = self as usize;
        let offset = match rotation {
            Rotation::Right => 1,
            Rotation::Left => 3, // +3 ≡ -1 (mod 4)
        };
        Self::ALL[(i + offset) % 4]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    Right,
    Left,
}

#[derive(Clone, Copy)]
pub struct PuyoPuyo {
    axis: Puyo,
    child: Puyo,
    orientation: Orientation,
}

impl PuyoPuyo {
    pub fn axis(&self) -> Puyo {
        self.axis
    }

    pub fn child(&self) -> Puyo {
        self.child
    }

    pub fn new() -> Self {
        let puyos = [
            Puyo::Red,
            Puyo::Blue,
            Puyo::Green,
            Puyo::Yellow,
            Puyo::Purple,
        ];
        let axis = puyos[rand::gen_range(0, puyos.len())];
        let child = puyos[rand::gen_range(0, puyos.len())];
        PuyoPuyo {
            axis,
            child,
            orientation: Orientation::Up,
        }
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}
