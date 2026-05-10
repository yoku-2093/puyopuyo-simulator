use crate::types::Puyo;
use macroquad::prelude::*;

// フィールドサイズ
pub const COLS: usize = 6; // 見えているフィールドの列数
pub const ROWS: usize = 12; // 見えているフィールドの行数

// 落下・移動の時間間隔（秒）
const DROP_INTERVAL: f64 = 0.5;
const MOVE_INTERVAL: f64 = 0.05;
const MOVE_REPEAT_DELAY: f64 = 0.2; // 初回入力からリピート開始までの猶予
const LOCK_DELAY: f64 = 0.50; // 接地から固定までの猶予
const LOCK_DELAY_FAST: f64 = 0.15; // 下キー押下時の固定猶予
const QUICK_TURN_WINDOW: f64 = 0.3;
const DROP_GRAVITY: f64 = 50.0; // ちぎり時の重力加速度（rows/s^2）
const DROP_GRAVITY_INITIAL: f64 = 5.0; // ちぎり開始時の初速（rows/s）
const DISPLAY_CHASE_RATE: f64 = 40.0; // 移動の表示位置追従速度
const ROTATION_CHASE_RATE: f64 = 20.0; // 回転の表示角度追従速度
const SQUASHING_ANIM_DURATION: f64 = 0.15; // 着地スカッシュの長さ
const SQUASHING_SQUASH_RATIO: f32 = 0.3; // 縦の潰れの最大量（0.3 = 30%縮む）
const BLINK_DURATION: f64 = 0.5; // 点滅の長さ
const SPARKLE_DURATION: f64 = 0.5; // キラキラの長さ
const SPARKLE_WAIT: f64 = 0.15; // パーティクル開始から落下開始までの待ち
const BLINK_COUNT: u32 = 5; // 点滅回数
const PARTICLE_COUNT: usize = 4; // 1ぷよあたりのパーティクル数
const PARTICLE_SPEED_MIN: f32 = 3.0; // パーティクルの最低速度（グリッド/秒）
const PARTICLE_SPEED_MAX: f32 = 8.0; // パーティクルの最高速度
const PARTICLE_GRAVITY: f32 = 15.0; // パーティクルの重力（グリッド/秒²）
const PARTICLE_SIZE_MIN: f32 = 0.08; // パーティクルの最小サイズ（グリッド単位）
const PARTICLE_SIZE_MAX: f32 = 0.18; // パーティクルの最大サイズ

const GHOST_ROWS: usize = 2; // 見えない幽霊行の数
const INITIAL_POSITION: Position = Position::new(2, GHOST_ROWS); // ぷよの初期出現位置
const TOTAL_ROWS: usize = ROWS + GHOST_ROWS; // 幽霊行を含むフィールド全体の行数

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

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}

/// PuyoPuyo を生成するファクトリ
pub struct PuyoPuyoFactory {
    num_colors: usize,
}

impl PuyoPuyoFactory {
    const ALL_COLORS: [Puyo; 5] = [
        Puyo::Red,
        Puyo::Blue,
        Puyo::Green,
        Puyo::Yellow,
        Puyo::Purple,
    ];

    pub fn new(num_colors: usize) -> Self {
        PuyoPuyoFactory { num_colors }
    }

    pub fn create(&self) -> PuyoPuyo {
        let puyos = &Self::ALL_COLORS[..self.num_colors];
        let axis = puyos[rand::gen_range(0, puyos.len())];
        let child = puyos[rand::gen_range(0, puyos.len())];
        PuyoPuyo {
            axis,
            child,
            orientation: Orientation::Up,
        }
    }
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

    pub fn to_angle(self) -> f64 {
        match self {
            Orientation::Up => -std::f64::consts::FRAC_PI_2,
            Orientation::Right => 0.0,
            Orientation::Down => std::f64::consts::FRAC_PI_2,
            Orientation::Left => std::f64::consts::PI,
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

/// 列番号（0 = 左端）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Col(usize);

impl Col {
    pub fn index(self) -> usize {
        self.0
    }
}

/// 幽霊行を含むフィールド全体での行番号（0 = 最上幽霊行）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FieldRow(usize);

/// 見える領域での行番号（0 = 見えるフィールドの最上行）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VisibleRow(usize);

impl FieldRow {
    #[allow(dead_code)]
    pub fn to_visible(self) -> VisibleRow {
        debug_assert!(self.0 >= GHOST_ROWS);
        VisibleRow(self.0 - GHOST_ROWS)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

impl VisibleRow {
    pub fn to_field(self) -> FieldRow {
        FieldRow(self.0 + GHOST_ROWS)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlayState {
    Active,    // 操作中
    Settling,  // 接地して固定待ち
    Dropping,  // ちぎり後の自由落下
    Squashing, // 着地直後でぷよが潰れるアニメ中
    Blinking,  // 点滅中
    Sparkling, // 弾けるアニメ中
    Landed,    // 接地完了
}

#[derive(Clone, Copy)]
pub struct DrawEffect {
    pub scale_x: f32,
    pub scale_y: f32,
    pub visible: bool,
}

impl Default for DrawEffect {
    fn default() -> Self {
        DrawEffect {
            scale_x: 1.0,
            scale_y: 1.0,
            visible: true,
        }
    }
}

impl DrawEffect {
    fn squash(mut self, progress: f32) -> Self {
        let squash = SQUASHING_SQUASH_RATIO * (std::f32::consts::PI * progress).sin();
        self.scale_x = 1.0 + squash * 0.5;
        self.scale_y = 1.0 - squash;
        self
    }

    fn blink(mut self, progress: f32) -> Self {
        let phase = (progress * BLINK_COUNT as f32 * 2.0) as i32;
        self.visible = phase % 2 == 0;
        self
    }
}

pub struct DrawPuyo {
    pub puyo: Puyo,
    pub col: f32,
    pub row: f32,
    pub effect: DrawEffect,
}

pub struct DroppingPuyo {
    pub puyo: Puyo,
    pub col: Col,
    pub row: f64,      // 現在の表示位置
    target_row: usize, // 着地する行
    velocity: f64,     // 落下速度（rows/s）
}

impl DroppingPuyo {
    pub fn new(puyo: Puyo, col: Col, row: f64, target_row: usize) -> Self {
        DroppingPuyo {
            puyo,
            col,
            row,
            target_row,
            velocity: DROP_GRAVITY_INITIAL,
        }
    }
}

pub struct SquashingPuyo {
    pub col: Col,
    pub row: FieldRow,
    pub start_time: f64,
}

pub struct BlinkingPuyo {
    pub col: Col,
    pub row: VisibleRow,
    pub start_time: f64,
}

pub struct SparklingPuyo {
    pub puyo: Puyo,
    pub col: Col,
    pub row: VisibleRow,
}

pub struct Particle {
    pub color: Color,
    pub col: f32,
    pub row: f32,
    vcol: f32,
    vrow: f32,
    pub size: f32,
    lifetime: f32,
    elapsed: f32,
}

impl Particle {
    pub fn alpha(&self) -> f32 {
        (1.0 - self.elapsed / self.lifetime).max(0.0)
    }

    fn alive(&self) -> bool {
        self.elapsed < self.lifetime
    }

    fn tick(&mut self, dt: f32) {
        self.elapsed += dt;
        self.col += self.vcol * dt;
        self.row += self.vrow * dt;
        self.vrow += PARTICLE_GRAVITY * dt;
    }
}

fn puyo_color(puyo: Puyo) -> Color {
    match puyo {
        Puyo::Red => Color::new(1.0, 0.2, 0.1, 1.0),
        Puyo::Blue => Color::new(0.0, 0.2, 1.0, 1.0),
        Puyo::Green => Color::new(0.0, 0.5, 0.15, 1.0),
        Puyo::Yellow => Color::new(1.0, 0.85, 0.0, 1.0),
        Puyo::Purple => Color::new(0.3, 0.0, 0.5, 1.0),
    }
}

// ゲームの状態を表す構造体
// positionのrowは幽霊行を含む行番号で管理する（例: position.row == 0 は最上幽霊行）
pub struct GameField {
    factory: PuyoPuyoFactory, // ぷよ生成用ファクトリ
    puyopuyo: PuyoPuyo, // 落下中のぷよ
    position: Position, // 軸ぷよの位置
    display_col: f64,   // 軸の表示位置（補間用）
    display_row: f64,
    display_angle: f64,                        // 子ぷよの表示角度（補間用）
    next: PuyoPuyo,                            // 次のぷよ
    next_next: PuyoPuyo,                       // 次の次のぷよ
    field: [[Option<Puyo>; COLS]; TOTAL_ROWS], // フィールド（幽霊行を含む）
    is_game_over: bool,
    score: u32,                    // スコア
    chain_count: u32,              // 現在の連鎖数
    dropping: Vec<DroppingPuyo>,   // ちぎり中のぷよ
    squashing: Vec<SquashingPuyo>, // 着地直後でぷよが潰れるアニメ中
    blinking: Vec<BlinkingPuyo>,   // 点滅中のぷよ
    sparkling: Vec<SparklingPuyo>, // 弾けるアニメ中
    particles: Vec<Particle>,      // パーティクル
    spawn_count: u32,              // スポーン回数
}

pub struct PlayContext {
    pub play_state: PlayState,
    last_drop_time: f64,
    last_frame_time: f64, // 直前の tick の時刻（dt 計算用）
    last_move_time: f64,
    move_repeating: bool, // リピート移動が開始されているか
    settling_start: f64,  // 接地待ち開始時刻
    sparkle_start: f64,   // パーティクル開始時刻
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
            sparkle_start: 0.0,
            last_failed_rotation: None,
        }
    }
}

// --- GameField: 生成・公開API ---

impl GameField {
    pub fn new(num_colors: usize) -> Self {
        let factory = PuyoPuyoFactory::new(num_colors);
        GameField {
            puyopuyo: factory.create(),
            position: INITIAL_POSITION,
            display_col: INITIAL_POSITION.col as f64,
            display_row: INITIAL_POSITION.row as f64,
            display_angle: Orientation::Up.to_angle(),
            next: factory.create(),
            next_next: factory.create(),
            field: [[None; COLS]; TOTAL_ROWS],
            is_game_over: false,
            score: 0,
            chain_count: 0,
            dropping: Vec::new(),
            squashing: Vec::new(),
            blinking: Vec::new(),
            sparkling: Vec::new(),
            particles: Vec::new(),
            spawn_count: 0,
            factory,
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_over
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn spawn_count(&self) -> u32 {
        self.spawn_count
    }

    pub fn next(&self) -> &PuyoPuyo {
        &self.next
    }

    pub fn next_next(&self) -> &PuyoPuyo {
        &self.next_next
    }
}

// --- GameField: 描画 ---

impl GameField {
    /// 描画用のぷよリストを返す
    pub fn draw_list(&self, ctx: &PlayContext, now: f64) -> Vec<DrawPuyo> {
        let mut list = Vec::new();

        // フィールドのぷよ
        let cells = self.visible_field();
        for row in 0..ROWS {
            for col in 0..COLS {
                if let Some(puyo) = cells[row][col] {
                    let c = Col(col);
                    let mut effect = DrawEffect::default();
                    if let Some(p) = self.squashing_progress(c, VisibleRow(row).to_field(), now) {
                        effect = effect.squash(p);
                    }
                    if let Some(p) = self.blinking_progress(c, VisibleRow(row), now) {
                        effect = effect.blink(p);
                    }
                    if !effect.visible {
                        continue;
                    }
                    list.push(DrawPuyo {
                        puyo,
                        col: col as f32,
                        row: row as f32,
                        effect,
                    });
                }
            }
        }

        // 操作中の組ぷよ
        if matches!(ctx.play_state, PlayState::Active | PlayState::Settling) {
            let child_col = self.display_col + self.display_angle.cos();
            let child_row = self.display_row + self.display_angle.sin();
            let pairs = [
                (self.puyopuyo.axis(), self.display_col, self.display_row),
                (self.puyopuyo.child(), child_col, child_row),
            ];
            for (puyo, col, row) in pairs {
                if row >= (GHOST_ROWS as f64) - 0.5 {
                    list.push(DrawPuyo {
                        puyo,
                        col: col as f32,
                        row: (row - GHOST_ROWS as f64) as f32,
                        effect: DrawEffect::default(),
                    });
                }
            }
        }

        // ちぎり中のぷよ
        for f in &self.dropping {
            if f.row >= (GHOST_ROWS as f64) - 0.5 {
                list.push(DrawPuyo {
                    puyo: f.puyo,
                    col: f.col.index() as f32,
                    row: (f.row - GHOST_ROWS as f64) as f32,
                    effect: DrawEffect::default(),
                });
            }
        }

        list
    }

    pub fn particle_list(&self) -> &[Particle] {
        &self.particles
    }

    fn squashing_progress(&self, col: Col, row: FieldRow, now: f64) -> Option<f32> {
        self.squashing.iter().find_map(|l| {
            if l.col == col && l.row == row {
                Some(((now - l.start_time) / SQUASHING_ANIM_DURATION).clamp(0.0, 1.0) as f32)
            } else {
                None
            }
        })
    }

    fn blinking_progress(&self, col: Col, row: VisibleRow, now: f64) -> Option<f32> {
        self.blinking.iter().find_map(|l| {
            if l.col == col && l.row == row {
                Some(((now - l.start_time) / BLINK_DURATION).clamp(0.0, 1.0) as f32)
            } else {
                None
            }
        })
    }
}

// --- GameField: tick（状態遷移） ---

impl GameField {
    pub fn tick(&mut self, ctx: &mut PlayContext, now: f64) {
        match ctx.play_state {
            PlayState::Active => self.tick_active(ctx, now),
            PlayState::Settling => self.tick_settling(ctx, now),
            PlayState::Dropping => self.tick_dropping(ctx, now),
            PlayState::Squashing => self.tick_squashing(ctx, now),
            PlayState::Blinking => self.tick_blinking(ctx, now),
            PlayState::Sparkling => self.tick_sparkling(ctx, now),
            PlayState::Landed => self.tick_landed(ctx, now),
        }
    }

    fn tick_active(&mut self, ctx: &mut PlayContext, now: f64) {
        if now - ctx.last_drop_time > DROP_INTERVAL {
            self.move_down();
            ctx.last_drop_time = now;
        }
        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx, now);
        if self.is_grounded() {
            ctx.play_state = PlayState::Settling;
            ctx.settling_start = now;
        }
    }

    fn tick_settling(&mut self, ctx: &mut PlayContext, now: f64) {
        let prev_col = self.position.col;
        let prev_row = self.position.row;
        let prev_ori = self.puyopuyo.orientation();

        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx, now);

        // 操作で位置や向きが変わったら猶予をリセット
        if self.position.col != prev_col
            || self.position.row != prev_row
            || self.puyopuyo.orientation() != prev_ori
        {
            ctx.settling_start = now;
        }

        let delay = if is_key_down(KeyCode::Down) {
            LOCK_DELAY_FAST
        } else {
            LOCK_DELAY
        };
        if !self.is_grounded() {
            ctx.play_state = PlayState::Active;
        } else if now - ctx.settling_start > delay
            && (self.display_row - self.position.row as f64).abs() < 0.05
        {
            let axis = self.position;
            let child = self.child_position();
            self.start_dropping(vec![
                (self.puyopuyo.axis(), Col(axis.col), FieldRow(axis.row)),
                (self.puyopuyo.child(), Col(child.col), FieldRow(child.row)),
            ]);
            ctx.play_state = PlayState::Dropping;
        }
    }

    fn tick_dropping(&mut self, ctx: &mut PlayContext, now: f64) {
        let mut i = 0;
        while i < self.dropping.len() {
            if self.dropping[i].row >= self.dropping[i].target_row as f64 {
                let f = self.dropping.remove(i);
                self.field[f.target_row][f.col.index()] = Some(f.puyo);
                self.squashing.push(SquashingPuyo {
                    col: f.col,
                    row: FieldRow(f.target_row),
                    start_time: now,
                });
            } else {
                i += 1;
            }
        }
        if self.dropping.is_empty() {
            ctx.play_state = PlayState::Squashing;
        }
    }

    fn tick_squashing(&mut self, ctx: &mut PlayContext, now: f64) {
        self.squashing
            .retain(|l| now - l.start_time < SQUASHING_ANIM_DURATION);
        if self.squashing.is_empty() {
            // 連鎖判定
            let groups = self.find_chain_groups();
            for (puyo, cells) in &groups {
                for &(col, row) in cells {
                    let vcol = Col(col);
                    let vrow = VisibleRow(row);
                    self.blinking.push(BlinkingPuyo {
                        col: vcol,
                        row: vrow,
                        start_time: now,
                    });
                    self.sparkling.push(SparklingPuyo {
                        puyo: *puyo,
                        col: vcol,
                        row: vrow,
                    });
                }
            }
            if groups.is_empty() {
                self.chain_count = 0;
                ctx.play_state = PlayState::Landed;
            } else {
                self.add_chain_score(&groups);
                ctx.play_state = PlayState::Blinking;
            }
        }
    }

    fn tick_blinking(&mut self, ctx: &mut PlayContext, now: f64) {
        self.blinking
            .retain(|l| now - l.start_time < BLINK_DURATION);
        if self.blinking.is_empty() {
            // フィールドからぷよを除去してパーティクルを生成
            let sparkling: Vec<_> = self.sparkling.drain(..).collect();
            for sp in sparkling {
                self.field[sp.row.to_field().index()][sp.col.index()] = None;
                self.spawn_particles(sp.puyo, sp.col, sp.row);
            }
            ctx.sparkle_start = now;
            ctx.play_state = PlayState::Sparkling;
        }
    }

    fn tick_sparkling(&mut self, ctx: &mut PlayContext, now: f64) {
        if now - ctx.sparkle_start >= SPARKLE_WAIT {
            let floating = self.collect_floating();
            if floating.is_empty() {
                ctx.play_state = PlayState::Landed;
            } else {
                self.start_dropping(floating);
                ctx.play_state = PlayState::Dropping;
            }
        }
    }

    fn tick_landed(&mut self, ctx: &mut PlayContext, now: f64) {
        self.chain_count = 0;

        if self.field[INITIAL_POSITION.row][INITIAL_POSITION.col].is_some() {
            self.is_game_over = true;
        }

        self.spawn_next();
        ctx.play_state = PlayState::Active;
        ctx.last_drop_time = now;
    }
}

// --- GameField: 操作（移動・回転） ---

impl GameField {
    fn child_position(&self) -> Position {
        let (dc, dr) = self.puyopuyo.orientation().offset();
        Position::new(
            (self.position.col as isize + dc) as usize,
            (self.position.row as isize + dr) as usize,
        )
    }

    fn is_grounded(&self) -> bool {
        !self.can_move((0, 1))
    }

    fn can_move(&self, (dc, dr): (isize, isize)) -> bool {
        let (child_dc, child_dr) = self.puyopuyo.orientation().offset();
        let new_pos = (
            self.position.col as isize + dc,
            self.position.row as isize + dr,
        );
        let new_child = (new_pos.0 + child_dc, new_pos.1 + child_dr);
        self.can_pass(new_pos) && self.can_pass(new_child)
    }

    fn can_pass(&self, (col, row): (isize, isize)) -> bool {
        col >= 0
            && col < COLS as isize
            && row >= 0
            && row < TOTAL_ROWS as isize
            && self.field[row as usize][col as usize].is_none()
    }

    fn move_left(&mut self) {
        if self.can_move((-1, 0)) {
            self.position.col -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.can_move((1, 0)) {
            self.position.col += 1;
        }
    }

    fn move_down(&mut self) -> bool {
        if self.can_move((0, 1)) {
            self.position.row += 1;
            true
        } else {
            false
        }
    }

    fn rotate(&mut self, rotation: Rotation, is_quick: bool) -> bool {
        let (col, row) = (self.position.col as isize, self.position.row as isize);
        let new_ori = if is_quick {
            self.puyopuyo
                .orientation()
                .rotate(rotation)
                .rotate(rotation)
        } else {
            self.puyopuyo.orientation().rotate(rotation)
        };
        let (new_dc, new_dr) = new_ori.offset();
        // キックの例（Up → Right 時計回り回転, A=軸 C=子 X=障害物 |=壁）
        //
        // 1. そのまま回転OK（new_child が空き）
        //    [C]
        //    [A]      →  [A][C]
        //
        // 2. 通常キック（new_child が壁、kick_to が空き → 軸を子の反対方向に移動）
        //    [C]|              |
        //    [A]|     →  [A][C]|    ※ 子が元の軸位置に来る
        //
        // 3. 上キック（new_child も kick_to も塞がれ → 軸を1段上に移動）
        //   |[C]          |[A][C]
        //   |[A][X]  →   |   [X]
        let new_child = (col + new_dc, row + new_dr);
        let kick_to = (col - new_dc, row - new_dr);

        if self.can_pass(new_child) {
            // そのまま回転OK
        } else if self.can_pass(kick_to) {
            // 通常キック（軸を反対方向に）
            self.position.col = kick_to.0 as usize;
            self.position.row = kick_to.1 as usize;
        } else {
            let up_axis = (col, row - 1);
            let up_child = (col + new_dc, row - 1 + new_dr);
            if !self.can_pass(up_axis) || !self.can_pass(up_child) {
                return false;
            }
            // 上キック（軸を1段上に）
            self.position.row -= 1;
        }
        self.puyopuyo.set_orientation(new_ori);
        true
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

    fn handle_rotate_keys(&mut self, ctx: &mut PlayContext, now: f64) {
        if is_key_pressed(KeyCode::X) {
            self.try_rotate(ctx, Rotation::Right, now);
        }
        if is_key_pressed(KeyCode::Z) {
            self.try_rotate(ctx, Rotation::Left, now);
        }
    }

    fn try_rotate(&mut self, ctx: &mut PlayContext, rotation: Rotation, now: f64) {
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

// --- GameField: フレーム更新（表示補間・落下物理・パーティクル） ---

impl GameField {
    pub fn update(&mut self, ctx: &mut PlayContext, now: f64) {
        let dt = (now - ctx.last_frame_time).clamp(0.0, 0.1);

        // 操作中のぷよの表示位置・角度を補間
        if matches!(ctx.play_state, PlayState::Active | PlayState::Settling) {
            let move_factor = (DISPLAY_CHASE_RATE * dt).min(1.0);
            self.display_col += (self.position.col as f64 - self.display_col) * move_factor;
            self.display_row += (self.position.row as f64 - self.display_row) * move_factor;

            let rot_factor = (ROTATION_CHASE_RATE * dt).min(1.0);
            let target = self.puyopuyo.orientation().to_angle();
            let mut diff = target - self.display_angle;
            if diff > std::f64::consts::PI {
                diff -= 2.0 * std::f64::consts::PI;
            }
            if diff < -std::f64::consts::PI {
                diff += 2.0 * std::f64::consts::PI;
            }
            self.display_angle += diff * rot_factor;
        }

        // 落下中のぷよの物理演算
        for f in &mut self.dropping {
            f.velocity += DROP_GRAVITY * dt;
            f.row += f.velocity * dt;
        }

        // パーティクル
        for p in &mut self.particles {
            p.tick(dt as f32);
        }
        self.particles.retain(|p| p.alive());

        ctx.last_frame_time = now;
    }
}

// --- GameField: フィールド操作（落下・連鎖・パーティクル） ---

impl GameField {
    /// 幽霊行を除いた見える部分のフィールドを返す
    pub fn visible_field(&self) -> &[[Option<Puyo>; COLS]] {
        &self.field[GHOST_ROWS..]
    }

    // ネクストぷよに切り替え
    fn spawn_next(&mut self) {
        self.spawn_count += 1;
        self.puyopuyo = self.next;
        self.next = self.next_next;
        self.next_next = self.factory.create();
        self.position = INITIAL_POSITION;
        self.display_col = INITIAL_POSITION.col as f64;
        self.display_row = INITIAL_POSITION.row as f64;
        self.display_angle = Orientation::Up.to_angle();
    }

    /// スコア加算: (消したぷよ数 × 10) × max(1, 連鎖ボーナス + 色ボーナス + グループボーナス)
    fn add_chain_score(&mut self, groups: &[(Puyo, Vec<(usize, usize)>)]) {
        self.chain_count += 1;
        let cleared: u32 = groups.iter().map(|(_, cells)| cells.len() as u32).sum();
        let cp = Self::chain_power(self.chain_count);
        let cb = Self::color_bonus(groups);
        let gb = Self::group_bonus(groups);
        let multiplier = (cp + cb + gb).clamp(1, 999);
        self.score += cleared * 10 * multiplier;
    }

    /// 連鎖数に応じた連鎖パワー
    fn chain_power(chain: u32) -> u32 {
        match chain {
            1 => 0,
            2 => 8,
            3 => 16,
            4 => 32,
            5 => 64,
            6 => 96,
            7 => 128,
            8 => 160,
            9 => 192,
            10 => 224,
            11 => 256,
            12 => 288,
            13 => 320,
            14 => 352,
            15 => 384,
            16 => 416,
            17 => 448,
            18 => 480,
            19 => 512,
            _ => 512 + (chain - 19) * 32,
        }
    }

    /// 同時に消えた色数に応じたボーナス
    fn color_bonus(groups: &[(Puyo, Vec<(usize, usize)>)]) -> u32 {
        let mut colors = std::collections::HashSet::new();
        for (puyo, _) in groups {
            colors.insert(*puyo);
        }
        match colors.len() {
            0 | 1 => 0,
            2 => 3,
            3 => 6,
            4 => 12,
            _ => 24,
        }
    }

    /// 各グループのサイズに応じたボーナスの合計
    fn group_bonus(groups: &[(Puyo, Vec<(usize, usize)>)]) -> u32 {
        groups
            .iter()
            .map(|(_, cells)| match cells.len() {
                0..=4 => 0,
                5 => 2,
                6 => 3,
                7 => 4,
                8 => 5,
                9 => 6,
                10 => 7,
                _ => 10,
            })
            .sum()
    }

    /// ぷよのリストを受け取り、着地先を計算して dropping に追加
    /// 落下不要（下に空白がない）なら直接フィールドに配置する
    fn start_dropping(&mut self, mut falling: Vec<(Puyo, Col, FieldRow)>) {
        // 同じ列内では下のぷよから先に着地先を割り当てる
        falling.sort_by(|a, b| {
            a.1.index()
                .cmp(&b.1.index())
                .then(b.2.index().cmp(&a.2.index()))
        });

        let mut offsets = [0usize; COLS];
        for (puyo, col, row) in falling {
            let ci = col.index();
            let Some(landing) = self.landing_row(col) else {
                continue;
            };
            let target = landing.index() - offsets[ci];
            if target <= row.index() {
                // 着地先が現在位置以上なら直接配置（上に落ちることはない）
                self.field[target][ci] = Some(puyo);
            } else {
                offsets[ci] += 1;
                self.dropping
                    .push(DroppingPuyo::new(puyo, col, row.index() as f64, target));
            }
        }
    }

    /// 指定列の着地可能な最下行を返す（全て埋まっている場合は None）
    fn landing_row(&self, col: Col) -> Option<FieldRow> {
        let ci = col.index();
        (0..TOTAL_ROWS)
            .rev()
            .find(|&r| self.field[r][ci].is_none())
            .map(FieldRow)
    }

    /// フィールドから浮いているぷよを集めて返す（フィールドからは除去される）
    fn collect_floating(&mut self) -> Vec<(Puyo, Col, FieldRow)> {
        let mut floating = Vec::new();
        for col in 0..COLS {
            let c = Col(col);
            let Some(bottom) = self.landing_row(c) else {
                continue;
            };
            for row in (0..bottom.index()).rev() {
                if let Some(puyo) = self.field[row][col].take() {
                    floating.push((puyo, c, FieldRow(row)));
                }
            }
        }
        floating
    }

    fn spawn_particles(&mut self, puyo: Puyo, col: Col, row: VisibleRow) {
        let color = puyo_color(puyo);
        for _ in 0..PARTICLE_COUNT {
            let angle = rand::gen_range(0.0f32, 2.0 * std::f32::consts::PI);
            let speed = rand::gen_range(PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX);
            self.particles.push(Particle {
                color,
                col: col.index() as f32,
                row: row.index() as f32,
                vcol: angle.cos() * speed,
                vrow: angle.sin() * speed,
                size: rand::gen_range(PARTICLE_SIZE_MIN, PARTICLE_SIZE_MAX),
                lifetime: SPARKLE_DURATION as f32,
                elapsed: 0.0,
            });
        }
    }

    /// 4個以上つながったグループを返す: Vec<(色, セル座標リスト)>
    fn find_chain_groups(&self) -> Vec<(Puyo, Vec<(usize, usize)>)> {
        let cells = self.visible_field();
        let mut visited = [[false; COLS]; ROWS];
        let mut groups = Vec::new();

        for row in 0..ROWS {
            for col in 0..COLS {
                if visited[row][col] {
                    continue;
                }
                let Some(puyo) = cells[row][col] else {
                    continue;
                };
                let mut group = Vec::new();
                Self::flood_fill(col, row, puyo, cells, &mut visited, &mut group);
                if group.len() >= 4 {
                    groups.push((puyo, group));
                }
            }
        }
        groups
    }

    fn flood_fill(
        col: usize,
        row: usize,
        target: Puyo,
        cells: &[[Option<Puyo>; COLS]],
        visited: &mut [[bool; COLS]; ROWS],
        group: &mut Vec<(usize, usize)>,
    ) {
        if visited[row][col] || cells[row][col] != Some(target) {
            return;
        }
        visited[row][col] = true;
        group.push((col, row));

        if row > 0 {
            Self::flood_fill(col, row - 1, target, cells, visited, group);
        }
        if row + 1 < ROWS {
            Self::flood_fill(col, row + 1, target, cells, visited, group);
        }
        if col > 0 {
            Self::flood_fill(col - 1, row, target, cells, visited, group);
        }
        if col + 1 < COLS {
            Self::flood_fill(col + 1, row, target, cells, visited, group);
        }
    }
}
