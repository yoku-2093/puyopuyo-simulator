use crate::types::Puyo;
use macroquad::prelude::*;

// フィールドサイズ
pub const COLS: usize = 6; // 見えているフィールドの列数
pub const ROWS: usize = 12; // 見えているフィールドの行数

// 落下・移動の時間間隔（秒）
const DROP_INTERVAL: f64 = 0.5;
const MOVE_INTERVAL: f64 = 0.05;
const MOVE_REPEAT_DELAY: f64 = 0.2; // 初回入力からリピート開始までの猶予
const LOCK_DELAY: f64 = 0.70; // 接地から固定までの猶予
const LOCK_DELAY_FAST: f64 = 0.18; // 下キー押下時の固定猶予
const QUICK_TURN_WINDOW: f64 = 0.5; // 同方向 2 連打でクイックターンを発動する猶予
const DROP_GRAVITY: f64 = 50.0; // ちぎり時の重力加速度（rows/s^2）
const DROP_GRAVITY_INITIAL: f64 = 5.0; // ちぎり開始時の初速（rows/s）
const DISPLAY_CHASE_RATE: f64 = 70.0; // 移動の表示位置追従速度
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

/// ゲーム中に発生するイベント（コントローラーが取り出して処理する）
pub enum GameEvent {
    PuyoLanded, // ぷよが設置された
    /// 連鎖が発生した（1 連鎖につき 1 回）。col/row は消えたぷよの重心 (visible grid)。
    ChainPop {
        count: u32,
        col: f32,
        row: f32,
    },
    GameOver, // ゲームオーバーになった
}

/// PuyoPuyo を生成するファクトリ。seed から決定的に色選択 / 個別ぷよ生成を行う。
pub struct PuyoPuyoFactory {
    colors: Vec<Puyo>,
    rng: Xorshift64,
}

/// 同じ seed を渡せば同じ列を再現できるための小さな xorshift PRNG。
struct Xorshift64(u64);

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        // 0 は xorshift で永久 0 になるので避ける
        Xorshift64(seed.max(1))
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn gen_range(&mut self, max: usize) -> usize {
        (self.next_u64() as usize) % max
    }
}

impl PuyoPuyoFactory {
    const ALL_COLORS: [Puyo; 5] = [
        Puyo::Red,
        Puyo::Blue,
        Puyo::Green,
        Puyo::Yellow,
        Puyo::Purple,
    ];

    pub fn new(num_colors: usize, seed: u64) -> Self {
        let mut rng = Xorshift64::new(seed);
        // ALL_COLORS から num_colors 個を選ぶ（Fisher-Yates）
        let mut colors = Self::ALL_COLORS.to_vec();
        for i in (1..colors.len()).rev() {
            let j = rng.gen_range(i + 1);
            colors.swap(i, j);
        }
        colors.truncate(num_colors);
        PuyoPuyoFactory { colors, rng }
    }

    pub fn create(&mut self) -> PuyoPuyo {
        let axis = self.colors[self.rng.gen_range(self.colors.len())];
        let child = self.colors[self.rng.gen_range(self.colors.len())];
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
    events: Vec<GameEvent>,   // 発生したイベント（コントローラーが drain する）
    puyopuyo: PuyoPuyo,       // 落下中のぷよ
    position: Position,       // 軸ぷよの位置
    display_col: f64,         // 軸の表示位置（補間用）
    display_row: f64,
    display_angle: f64,   // 子ぷよの表示角度（補間用、現在の見た目）
    rotation_target: f64, // 子ぷよの目標角度（累積、wrap なし）
    next: PuyoPuyo,       // 次のぷよ
    next_next: PuyoPuyo,  // 次の次のぷよ
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
    last_rotation_press: Option<(Rotation, f64)>, // クイックターン用: 直近の回転キー押下
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
            last_rotation_press: None,
        }
    }
}

pub struct GameInput {
    pub left: bool,
    pub right: bool,
    pub down: bool,
    pub left_just: bool,
    pub right_just: bool,
    pub down_just: bool,
    pub rotate_right: bool,
    pub rotate_left: bool,
}

// --- GameField: 生成・公開API ---

impl GameField {
    pub fn new(num_colors: usize, seed: u64) -> Self {
        let mut factory = PuyoPuyoFactory::new(num_colors, seed);
        GameField {
            puyopuyo: factory.create(),
            position: INITIAL_POSITION,
            display_col: INITIAL_POSITION.col as f64,
            display_row: INITIAL_POSITION.row as f64,
            display_angle: Orientation::Up.to_angle(),
            rotation_target: Orientation::Up.to_angle(),
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
            events: Vec::new(),
        }
    }

    /// 発生したイベントを取り出す（取り出し後はクリア）
    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
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
    pub fn tick(&mut self, ctx: &mut PlayContext, now: f64, input: &GameInput) {
        match ctx.play_state {
            PlayState::Active => self.tick_active(ctx, now, input),
            PlayState::Settling => self.tick_settling(ctx, now, input),
            PlayState::Dropping => self.tick_dropping(ctx, now),
            PlayState::Squashing => self.tick_squashing(ctx, now),
            PlayState::Blinking => self.tick_blinking(ctx, now),
            PlayState::Sparkling => self.tick_sparkling(ctx, now),
            PlayState::Landed => self.tick_landed(ctx, now),
        }
    }

    fn tick_active(&mut self, ctx: &mut PlayContext, now: f64, input: &GameInput) {
        if now - ctx.last_drop_time > DROP_INTERVAL {
            self.move_down();
            ctx.last_drop_time = now;
        }
        self.handle_move_keys(ctx, now, input);
        self.handle_rotate_keys(ctx, now, input);
        if self.is_grounded() {
            ctx.play_state = PlayState::Settling;
            ctx.settling_start = now;
        }
    }

    fn tick_settling(&mut self, ctx: &mut PlayContext, now: f64, input: &GameInput) {
        let prev_col = self.position.col;
        let prev_row = self.position.row;
        let prev_ori = self.puyopuyo.orientation();

        self.handle_move_keys(ctx, now, input);
        self.handle_rotate_keys(ctx, now, input);

        // 操作で位置や向きが変わったら猶予をリセット
        if self.position.col != prev_col
            || self.position.row != prev_row
            || self.puyopuyo.orientation() != prev_ori
        {
            ctx.settling_start = now;
        }

        let delay = if input.down {
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
                self.events.push(GameEvent::PuyoLanded);
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
                // 消えたぷよ全体の重心を計算してイベントに載せる
                let cells = groups.iter().flat_map(|(_, c)| c.iter().copied());
                let (mut sum_c, mut sum_r, mut n) = (0.0f32, 0.0f32, 0u32);
                for (c, r) in cells {
                    sum_c += c as f32;
                    sum_r += r as f32;
                    n += 1;
                }
                self.events.push(GameEvent::ChainPop {
                    count: self.chain_count,
                    col: sum_c / n as f32,
                    row: sum_r / n as f32,
                });
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
            self.events.push(GameEvent::GameOver);
            return;
        }

        self.spawn_next();
        ctx.play_state = PlayState::Active;
        ctx.last_drop_time = now;
        ctx.move_repeating = false;
        ctx.last_rotation_press = None;
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
        let new_axis = (
            self.position.col as isize + dc,
            self.position.row as isize + dr,
        );
        let new_child = (new_axis.0 + child_dc, new_axis.1 + child_dr);
        self.can_pass(new_axis, new_child)
    }

    /// 軸と子の組がその位置に居られるか判定する。
    /// - 軸 (axis): 画面内 (横+上下) かつ空きセル。
    /// - 子 (child): 列は画面内必須。行は上端より上 (row < 0) は素通しOK
    ///   (最上段で Up に回転した時にキックで下に押し戻されるのを防ぐため)。
    fn can_pass(&self, axis: (isize, isize), child: (isize, isize)) -> bool {
        let (ac, ar) = axis;
        let (cc, cr) = child;
        let cols = COLS as isize;
        let total_rows = TOTAL_ROWS as isize;

        let axis_ok = ac >= 0
            && ac < cols
            && ar >= 0
            && ar < total_rows
            && self.field[ar as usize][ac as usize].is_none();

        let child_ok = cc >= 0
            && cc < cols
            && (cr < 0 || (cr < total_rows && self.field[cr as usize][cc as usize].is_none()));

        axis_ok && child_ok
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

        // === 1. まず直接回転を試す ===
        // 軸位置はそのまま、子だけ新しい位置に動かす。
        // Up に回す場合は can_pass 側で子の画面外上を許容しているため、
        // 軸が境界内である限りここで必ず成功する。
        if self.can_pass((col, row), (col + new_dc, row + new_dr)) {
            self.puyopuyo.set_orientation(new_ori);
            return true;
        }

        // === 2. 直接が失敗 → 向きに応じてキックを試す ===
        //
        // [図解]  A=軸 C=子 X=障害物 |=壁
        //
        //   通常キック (軸を新しい子の反対方向にずらす)
        //      [C]|              |
        //      [A]|     →  [A][C]|
        //
        //   上キック (軸を1段上にずらす)
        //     |[C]         |[A][C]
        //     |[A][X] →   |   [X]
        //
        //   斜めキック (通常 + 上 を同時に)
        //        [C][X]          [A][C][X]
        //     [X][A][X]    →    [X]   [X]
        let kicks: &[(isize, isize)] = match new_ori {
            // Up は直接回転で必ず成功する (子の画面外上を can_pass が許容し、軸の真上に
            // 静止ぷよが浮くのは重力的に不可能なため)。よってここに来た時点で不変条件違反。
            // 開発中はパニックで気付き、本番では黙って回転失敗にとどめる。
            Orientation::Up => {
                debug_assert!(false, "Up rotation must succeed via direct rotation");
                return false;
            }
            // Down は上キックでだけ救済できる。
            Orientation::Down => &[(0, -1)],
            // Right/Left は壁際・段差込みのケースを通常/上/斜めの3種で救う。
            Orientation::Right | Orientation::Left => &[
                (-new_dc, -new_dr),     // 通常キック
                (0, -1),                // 上キック
                (-new_dc, -new_dr - 1), // 斜めキック
            ],
        };

        for &(axis_dc, axis_dr) in kicks {
            let new_axis = (col + axis_dc, row + axis_dr);
            let new_child = (new_axis.0 + new_dc, new_axis.1 + new_dr);
            if self.can_pass(new_axis, new_child) {
                self.position.col = new_axis.0 as usize;
                self.position.row = new_axis.1 as usize;
                self.puyopuyo.set_orientation(new_ori);
                return true;
            }
        }
        false
    }

    fn handle_move_keys(&mut self, ctx: &mut PlayContext, now: f64, input: &GameInput) {
        let held = input.left || input.right || input.down;

        if !held {
            ctx.move_repeating = false;
            return;
        }

        let just_pressed = input.left_just || input.right_just || input.down_just;

        let interval = if ctx.move_repeating {
            MOVE_INTERVAL
        } else {
            MOVE_REPEAT_DELAY
        };
        if !just_pressed && now - ctx.last_move_time <= interval {
            return;
        }

        if input.left {
            self.move_left();
        }
        if input.right {
            self.move_right();
        }
        if input.down {
            self.move_down();
        }
        ctx.last_move_time = now;
        ctx.move_repeating = !just_pressed;
    }

    fn handle_rotate_keys(&mut self, ctx: &mut PlayContext, now: f64, input: &GameInput) {
        if input.rotate_right {
            self.try_rotate(ctx, Rotation::Right, now);
        }
        if input.rotate_left {
            self.try_rotate(ctx, Rotation::Left, now);
        }
    }

    /// クイックターン候補位置: 縦向きで左右どちらにも横移動できない。
    fn in_quick_turn_position(&self) -> bool {
        matches!(
            self.puyopuyo.orientation(),
            Orientation::Up | Orientation::Down
        ) && !self.can_move((-1, 0))
            && !self.can_move((1, 0))
    }

    fn try_rotate(&mut self, ctx: &mut PlayContext, rotation: Rotation, now: f64) {
        // クイックターンは「縦向き + 両サイド塞がれ」かつ「同方向 2 連打」のときだけ。
        let recent_same = matches!(
            ctx.last_rotation_press,
            Some((r, t)) if r == rotation && now - t < QUICK_TURN_WINDOW
        );
        let is_quick = recent_same && self.in_quick_turn_position();

        if self.rotate(rotation, is_quick) {
            // 表示用の累積角度を進める (wrap させずに方向を保持)
            let steps = if is_quick { 2.0 } else { 1.0 };
            let delta = match rotation {
                Rotation::Right => std::f64::consts::FRAC_PI_2 * steps,
                Rotation::Left => -std::f64::consts::FRAC_PI_2 * steps,
            };
            self.rotation_target += delta;
        }
        ctx.last_rotation_press = Some((rotation, now));
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
            // rotation_target は単調変化する累積角度なので wrap 不要。
            // 連打でも回転方向が反転しない。
            self.display_angle += (self.rotation_target - self.display_angle) * rot_factor;
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
        self.rotation_target = Orientation::Up.to_angle();
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

    /// 連鎖数 (1-indexed) に応じた連鎖パワー (ぷよぷよ通の公式テーブル)
    fn chain_power(chain: u32) -> u32 {
        const TABLE: [u32; 19] = [
            //  1   2   3   4   5    6    7    8    9   10
            0, 8, 16, 32, 64, 96, 128, 160, 192, 224,
            // 11  12  13  14   15   16   17   18   19
            256, 288, 320, 352, 384, 416, 448, 480, 512,
        ];
        if chain == 0 {
            return 0;
        }
        let idx = chain as usize - 1;
        if idx < TABLE.len() {
            TABLE[idx]
        } else {
            // 20 連鎖以降は +32 ずつ
            TABLE[TABLE.len() - 1] + (chain - TABLE.len() as u32) * 32
        }
    }

    /// 同時に消えた色数に応じたボーナス
    fn color_bonus(groups: &[(Puyo, Vec<(usize, usize)>)]) -> u32 {
        // index = 色数
        const TABLE: [u32; 6] = [0, 0, 3, 6, 12, 24];
        let mut colors = std::collections::HashSet::new();
        for (puyo, _) in groups {
            colors.insert(*puyo);
        }
        let n = colors.len().min(TABLE.len() - 1);
        TABLE[n]
    }

    /// 各グループのサイズに応じたボーナスの合計
    fn group_bonus(groups: &[(Puyo, Vec<(usize, usize)>)]) -> u32 {
        // index = グループのサイズ。0..=4 は 0、5 以降は加算、11+ は 10 固定
        const TABLE: [u32; 11] = [0, 0, 0, 0, 0, 2, 3, 4, 5, 6, 7];
        groups
            .iter()
            .map(|(_, cells)| {
                let n = cells.len();
                if n < TABLE.len() { TABLE[n] } else { 10 }
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
                self.events.push(GameEvent::PuyoLanded);
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
