use crate::constants::*;
use macroquad::prelude::*;

const GHOST_ROWS: usize = 2;
const INITIAL_POSITION: Position = Position::new(2, GHOST_ROWS);
const TOTAL_ROWS: usize = ROWS + GHOST_ROWS;

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
    pub col: usize,
    pub row: f64,      // 現在の表示位置
    target_row: usize, // 着地する行
    velocity: f64,     // 落下速度（rows/s）
}

impl DroppingPuyo {
    pub fn new(puyo: Puyo, col: usize, row: f64, target_row: usize) -> Self {
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
    pub col: usize,
    pub row: FieldRow,
    pub start_time: f64,
}

pub struct BlinkingPuyo {
    pub col: usize,
    pub row: VisibleRow,
    pub start_time: f64,
}

pub struct SparklingPuyo {
    pub puyo: Puyo,
    pub col: usize,
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
    puyopuyo: PuyoPuyo, // 落下中のぷよ
    position: Position, // 軸ぷよの位置
    display_col: f64,   // 軸の表示位置（補間用）
    display_row: f64,
    display_angle: f64,                        // 子ぷよの表示角度（補間用）
    next: PuyoPuyo,                            // 次のぷよ
    next_next: PuyoPuyo,                       // 次の次のぷよ
    field: [[Option<Puyo>; COLS]; TOTAL_ROWS], // フィールド（幽霊行を含む）
    is_game_over: bool,
    dropping: Vec<DroppingPuyo>,   // ちぎり中のぷよ
    squashing: Vec<SquashingPuyo>, // 着地直後でぷよが潰れるアニメ中
    blinking: Vec<BlinkingPuyo>,   // 点滅中のぷよ
    sparkling: Vec<SparklingPuyo>, // 弾けるアニメ中
    particles: Vec<Particle>,      // パーティクル
}

impl GameField {
    pub fn new() -> Self {
        GameField {
            puyopuyo: PuyoPuyo::new(),
            position: INITIAL_POSITION,
            display_col: INITIAL_POSITION.col as f64,
            display_row: INITIAL_POSITION.row as f64,
            display_angle: Orientation::Up.to_angle(),
            next: PuyoPuyo::new(),
            next_next: PuyoPuyo::new(),
            field: [[None; COLS]; TOTAL_ROWS],
            is_game_over: false,
            dropping: Vec::new(),
            squashing: Vec::new(),
            blinking: Vec::new(),
            sparkling: Vec::new(),
            particles: Vec::new(),
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_over
    }

    fn child_position(&self) -> Position {
        let (dc, dr) = self.puyopuyo.orientation().offset();
        Position::new(
            (self.position.col as isize + dc) as usize,
            (self.position.row as isize + dr) as usize,
        )
    }

    /// 接地しているか（下に動かせないか）
    fn is_grounded(&self) -> bool {
        !self.can_move((0, 1))
    }

    /// 幽霊行を除いた見える部分のフィールドを返す
    pub fn visible_field(&self) -> &[[Option<Puyo>; COLS]] {
        &self.field[GHOST_ROWS..]
    }

    /// 移動後の軸と子の両方が範囲内かつ空きマスか判定
    fn can_move(&self, (dc, dr): (isize, isize)) -> bool {
        let (child_dc, child_dr) = self.puyopuyo.orientation().offset();
        let new_pos = (
            self.position.col as isize + dc,
            self.position.row as isize + dr,
        );
        let new_child = (new_pos.0 + child_dc, new_pos.1 + child_dr);
        self.can_pass(new_pos) && self.can_pass(new_child)
    }

    /// 操作中のぷよが通過できるか（最上幽霊行は空なら通過可能）
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

    // ネクストぷよに切り替え
    fn spawn_next(&mut self) {
        self.puyopuyo = self.next;
        self.next = self.next_next;
        self.next_next = PuyoPuyo::new();
        self.position = INITIAL_POSITION;
        self.display_col = INITIAL_POSITION.col as f64;
        self.display_row = INITIAL_POSITION.row as f64;
        self.display_angle = Orientation::Up.to_angle();
    }

    /// 描画用のぷよリストを返す
    pub fn draw_list(&self, ctx: &PlayContext, now: f64) -> Vec<DrawPuyo> {
        let mut list = Vec::new();

        // フィールドのぷよ
        let cells = self.visible_field();
        for row in 0..ROWS {
            for col in 0..COLS {
                if let Some(puyo) = cells[row][col] {
                    let mut effect = DrawEffect::default();
                    if let Some(p) = self.squashing_progress(col, VisibleRow(row).to_field(), now) {
                        effect = effect.squash(p);
                    }
                    if let Some(p) = self.blinking_progress(col, VisibleRow(row), now) {
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
                    col: f.col as f32,
                    row: (f.row - GHOST_ROWS as f64) as f32,
                    effect: DrawEffect::default(),
                });
            }
        }

        list
    }

    fn squashing_progress(&self, col: usize, row: FieldRow, now: f64) -> Option<f32> {
        self.squashing.iter().find_map(|l| {
            if l.col == col && l.row == row {
                Some(((now - l.start_time) / SQUASHING_ANIM_DURATION).clamp(0.0, 1.0) as f32)
            } else {
                None
            }
        })
    }

    fn blinking_progress(&self, col: usize, row: VisibleRow, now: f64) -> Option<f32> {
        self.blinking.iter().find_map(|l| {
            if l.col == col && l.row == row {
                Some(((now - l.start_time) / BLINK_DURATION).clamp(0.0, 1.0) as f32)
            } else {
                None
            }
        })
    }

    pub fn particle_list(&self) -> &[Particle] {
        &self.particles
    }

    fn spawn_particles(&mut self, puyo: Puyo, col: usize, row: VisibleRow) {
        let color = puyo_color(puyo);
        for _ in 0..PARTICLE_COUNT {
            let angle = rand::gen_range(0.0f32, 2.0 * std::f32::consts::PI);
            let speed = rand::gen_range(PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX);
            self.particles.push(Particle {
                color,
                col: col as f32,
                row: row.index() as f32,
                vcol: angle.cos() * speed,
                vrow: angle.sin() * speed,
                size: rand::gen_range(PARTICLE_SIZE_MIN, PARTICLE_SIZE_MAX),
                lifetime: SPARKLE_DURATION as f32,
                elapsed: 0.0,
            });
        }
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
    /// 戻り値: ゲームオーバーなら false
    pub fn tick(&mut self, ctx: &mut PlayContext, now: f64) {
        let dt = (now - ctx.last_frame_time).clamp(0.0, 0.1);
        ctx.last_frame_time = now;
        match ctx.play_state {
            PlayState::Active => self.tick_active(ctx, now, dt),
            PlayState::Settling => self.tick_settling(ctx, now, dt),
            PlayState::Dropping => self.tick_dropping(ctx, now, dt),
            PlayState::Squashing => self.tick_squashing(ctx, now),
            PlayState::Blinking => self.tick_blinking(ctx, now),
            PlayState::Sparkling => self.tick_sparkling(ctx, now, dt),
            PlayState::Landed => self.tick_landed(ctx, now), // ゲームオーバー判定があるので特別扱い
        }
    }

    fn tick_active(&mut self, ctx: &mut PlayContext, now: f64, dt: f64) {
        if now - ctx.last_drop_time > DROP_INTERVAL {
            self.move_down();
            ctx.last_drop_time = now;
        }
        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx, now);
        self.update_display(dt);
        if self.is_grounded() {
            ctx.play_state = PlayState::Settling;
            ctx.settling_start = now;
        }
    }

    fn tick_settling(&mut self, ctx: &mut PlayContext, now: f64, dt: f64) {
        let prev_col = self.position.col;
        let prev_row = self.position.row;
        let prev_ori = self.puyopuyo.orientation();

        self.handle_move_keys(ctx, now);
        self.handle_rotate_keys(ctx, now);
        self.update_display(dt);

        // 操作で位置や向きが変わったら猶予をリセット
        if self.position.col != prev_col
            || self.position.row != prev_row
            || self.puyopuyo.orientation() != prev_ori
        {
            ctx.settling_start = now;
        }

        if !self.is_grounded() {
            ctx.play_state = PlayState::Active;
        } else if now - ctx.settling_start > LOCK_DELAY
            && (self.display_row - self.position.row as f64).abs() < 0.05
        {
            self.start_dropping();
            ctx.play_state = PlayState::Dropping;
        }
    }

    fn tick_dropping(&mut self, ctx: &mut PlayContext, now: f64, dt: f64) {
        let mut i = 0;
        while i < self.dropping.len() {
            let f = &mut self.dropping[i];
            f.velocity += DROP_GRAVITY * dt;
            f.row += f.velocity * dt;
            if f.row >= f.target_row as f64 {
                let f = self.dropping.remove(i);
                self.field[f.target_row][f.col] = Some(f.puyo);
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
            let chained: [[Option<Puyo>; COLS]; ROWS] = self.chained_puyos();
            for row in 0..ROWS {
                for col in 0..COLS {
                    if let Some(puyo) = chained[row][col] {
                        let vrow = VisibleRow(row);
                        self.blinking.push(BlinkingPuyo {
                            col,
                            row: vrow,
                            start_time: now,
                        });
                        self.sparkling.push(SparklingPuyo {
                            puyo,
                            col,
                            row: vrow,
                        });
                    }
                }
            }
            if self.blinking.is_empty() {
                ctx.play_state = PlayState::Landed;
            } else {
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
                self.field[sp.row.to_field().index()][sp.col] = None;
                self.spawn_particles(sp.puyo, sp.col, sp.row);
            }
            ctx.play_state = PlayState::Sparkling;
        }
    }

    fn tick_sparkling(&mut self, ctx: &mut PlayContext, _now: f64, dt: f64) {
        for p in &mut self.particles {
            p.tick(dt as f32);
        }
        self.particles.retain(|p| p.alive());
        if self.particles.is_empty() {
            ctx.play_state = PlayState::Landed;
        }
    }

    fn tick_landed(&mut self, ctx: &mut PlayContext, now: f64) {
        if self.field[INITIAL_POSITION.row][INITIAL_POSITION.col].is_some() {
            self.is_game_over = true;
        }

        self.spawn_next();
        ctx.play_state = PlayState::Active;
        ctx.last_drop_time = now;
    }

    /// 表示位置・角度を論理値に向かって補間
    fn update_display(&mut self, dt: f64) {
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

    /// 組ぷよを dropping に積んでちぎりを開始
    fn start_dropping(&mut self) {
        let axis_col = self.position.col;
        let axis_row = self.position.row;
        let child = self.child_position();
        let axis_puyo = self.puyopuyo.axis();
        let child_puyo = self.puyopuyo.child();

        if axis_col == child.col {
            // 同じ列: 下のぷよから順に target を割り当てる
            let col = axis_col;
            let mut bottom = self.bottom_empty(col);
            let (lower_puyo, lower_row, upper_puyo, upper_row) = if axis_row > child.row {
                (axis_puyo, axis_row, child_puyo, child.row)
            } else {
                (child_puyo, child.row, axis_puyo, axis_row)
            };
            self.dropping
                .push(DroppingPuyo::new(lower_puyo, col, lower_row as f64, bottom));
            bottom = bottom.saturating_sub(1);
            self.dropping
                .push(DroppingPuyo::new(upper_puyo, col, upper_row as f64, bottom));
        } else {
            // 別の列: 独立に target を計算
            let axis_target = self.bottom_empty(axis_col);
            let child_target = self.bottom_empty(child.col);
            self.dropping.push(DroppingPuyo::new(
                axis_puyo,
                axis_col,
                axis_row as f64,
                axis_target,
            ));
            self.dropping.push(DroppingPuyo::new(
                child_puyo,
                child.col,
                child.row as f64,
                child_target,
            ));
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

    fn chained_puyos(&self) -> [[Option<Puyo>; COLS]; ROWS] {
        let mut chained_field = [[None; COLS]; ROWS];
        let cells = self.visible_field();
        let mut visited = [[false; COLS]; ROWS];

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
                    for (c, r) in &group {
                        chained_field[*r][*c] = cells[*r][*c];
                    }
                }
            }
        }
        chained_field
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
