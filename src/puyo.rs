use macroquad::rand;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuyoColor {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}

#[derive(Clone, Copy)]
struct Puyo {
    color: PuyoColor,
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
pub struct KumiPuyo {
    axis: Puyo,
    child: Puyo,
    orientation: Orientation,
}

impl KumiPuyo {
    pub fn axis_color(&self) -> PuyoColor {
        self.axis.color
    }

    pub fn child_color(&self) -> PuyoColor {
        self.child.color
    }

    pub fn new() -> Self {
        let colors = [
            PuyoColor::Red,
            PuyoColor::Blue,
            PuyoColor::Green,
            PuyoColor::Yellow,
            PuyoColor::Purple,
        ];
        let axis_color = colors[rand::gen_range(0, colors.len())];
        let child_color = colors[rand::gen_range(0, colors.len())];
        KumiPuyo {
            axis: Puyo { color: axis_color },
            child: Puyo { color: child_color },
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
