//! Fixed-step dungeon simulation. Graphics and browser code live in the binary.
pub mod level;
use level::{COLS, Level, ROOM_H, ROOM_W, TILE_H, TILE_W, Tile, floor_y, tile_center};

pub const DT: f32 = 1.0 / 60.0;
const HEIGHT: f32 = 38.0;
const HALF_WIDTH: f32 = 6.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub action: bool,
    pub parry: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Idle,
    Running,
    Stopping,
    Turning,
    Stepping,
    Crouching,
    Jumping,
    Falling,
    Hanging,
    Climbing,
    Combat,
    Attacking,
    Parrying,
    Hurt,
    Dead,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Dead,
    Complete,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Step,
    Jump,
    Land,
    Grab,
    Gate,
    Crumble,
    Sword,
    Drink,
    Clash,
    Parry,
    Hit,
    Hurt,
    Death,
    Complete,
}
#[derive(Debug, Clone)]
pub struct Player {
    pub room: u8,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: i8,
    pub state: PlayerState,
    pub health: u8,
    pub sword: bool,
    pub grounded: bool,
    pub state_ticks: u16,
    pub invulnerable: u16,
    pub landing_ticks: u8,
    fall_distance: f32,
    hang_row: usize,
    hang_room: u8,
    hang_x: f32,
    climb_start_x: f32,
    climb_start_y: f32,
    coyote_ticks: u8,
    jump_buffer: u8,
}
#[derive(Debug, Clone)]
pub struct Guard {
    pub room: u8,
    pub x: f32,
    pub y: f32,
    pub facing: i8,
    pub health: u8,
    pub windup: u16,
    pub cooldown: u16,
    pub hurt: u16,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct TileRuntime {
    pub removed: bool,
    pub consumed: bool,
    pub loose_ticks: u16,
    pub gate_ticks: u16,
    pub gate_open: f32,
    pub pressed: bool,
}
#[derive(Debug, Clone)]
pub struct Game {
    pub level: Level,
    pub player: Player,
    pub guards: Vec<Guard>,
    pub tiles: Vec<[TileRuntime; 30]>,
    pub phase: Phase,
    pub tick: u64,
    pub elapsed_ticks: u32,
    pub events: Vec<Event>,
    pub combat_feedback_ticks: u8,
    pub last_combat_event: Event,
    previous: Input,
    attack_hit: bool,
    attack_buffer: u8,
    safe_floor: Option<(u8, usize)>,
}
impl Game {
    pub fn new(level: Level) -> Self {
        let start = level.start;
        let guards = level
            .rooms
            .iter()
            .filter_map(|room| {
                room.guard.map(|g| Guard {
                    room: room.id,
                    x: tile_center(g.tile),
                    y: floor_y(g.tile / COLS),
                    facing: g.facing,
                    health: 3,
                    windup: 0,
                    cooldown: 55,
                    hurt: 0,
                })
            })
            .collect();
        let tiles = vec![[TileRuntime::default(); 30]; level.rooms.len()];
        Self {
            level,
            player: Player {
                room: start.room,
                x: tile_center(start.tile),
                y: floor_y(start.tile / COLS),
                vx: 0.0,
                vy: 0.0,
                facing: start.facing,
                state: PlayerState::Falling,
                health: 3,
                sword: false,
                grounded: false,
                state_ticks: 0,
                invulnerable: 0,
                landing_ticks: 0,
                fall_distance: 0.0,
                hang_row: 0,
                hang_room: start.room,
                hang_x: 0.0,
                climb_start_x: 0.0,
                climb_start_y: 0.0,
                coyote_ticks: 0,
                jump_buffer: 0,
            },
            guards,
            tiles,
            phase: Phase::Playing,
            tick: 0,
            elapsed_ticks: 0,
            events: Vec::with_capacity(8),
            combat_feedback_ticks: 0,
            last_combat_event: Event::Clash,
            previous: Input::default(),
            attack_hit: false,
            attack_buffer: 0,
            safe_floor: None,
        }
    }
    pub fn restart(&mut self) {
        *self = Self::new(self.level.clone());
    }
    /// A repeatable sword-training encounter; the campaign still starts unarmed.
    pub fn start_duel(&mut self) {
        self.restart();
        if let Some(guard) = self.guards.first() {
            self.player.room = guard.room;
            self.player.x = guard.x - 44.0;
            self.player.y = guard.y;
            self.player.facing = 1;
            self.player.sword = true;
            self.player.grounded = true;
            self.player.state = PlayerState::Combat;
        }
    }
    pub fn runtime(&self, room: u8, tile: usize) -> TileRuntime {
        self.tiles[usize::from(room) - 1][tile]
    }
    pub fn tile(&self, room: u8, tile: usize) -> Tile {
        if self.runtime(room, tile).removed {
            Tile::Empty
        } else {
            self.level.tile(room, tile)
        }
    }
    fn set_state(&mut self, state: PlayerState) {
        if self.player.state != state {
            self.player.state = state;
            self.player.state_ticks = 0;
        }
    }
    pub fn floor_index(&self) -> Option<usize> {
        let p = &self.player;
        let row = ((p.y + 3.0) / TILE_H).round() as i32 - 1;
        let col = (p.x / TILE_W).floor() as i32;
        if (0..3).contains(&row) && (0..10).contains(&col) {
            Some(row as usize * COLS + col as usize)
        } else {
            None
        }
    }
    fn horizontal_point(&self, room: u8, x: f32) -> Option<(u8, f32)> {
        if x < 0.0 {
            let next = self.level.room(room).links[0];
            (next != 0).then_some((next, x + ROOM_W))
        } else if x >= ROOM_W {
            let next = self.level.room(room).links[1];
            (next != 0).then_some((next, x - ROOM_W))
        } else {
            Some((room, x))
        }
    }
    fn support(&self, room: u8, x: f32, y: f32) -> bool {
        let Some((room, x)) = self.horizontal_point(room, x) else {
            return false;
        };
        (0..3).any(|r| {
            (floor_y(r) - y).abs() < 1.0
                && self.tile(room, r * COLS + (x / TILE_W) as usize).supports()
        })
    }
    fn blocked(&self, room: u8, x: f32, y: f32) -> bool {
        let Some((room, x)) = self.horizontal_point(room, x) else {
            return true;
        };
        let col = (x / TILE_W) as usize;
        for row in 0..3 {
            let top = floor_y(row) - TILE_H;
            if y - 3.0 > top + 1.0 && y - HEIGHT + 5.0 < floor_y(row) - 2.0 {
                let idx = row * COLS + col;
                if self.tile(room, idx) == Tile::Wall {
                    return true;
                }
                if self.tile(room, idx) == Tile::Gate && self.runtime(room, idx).gate_open < 0.82 {
                    let gate_x = col as f32 * TILE_W + 25.0;
                    if (x - gate_x).abs() < 7.0 {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn hurt(&mut self, amount: u8) {
        if self.player.invulnerable > 0 || self.phase != Phase::Playing {
            return;
        }
        self.player.health = self.player.health.saturating_sub(amount);
        self.player.invulnerable = 65;
        self.events.push(Event::Hurt);
        self.set_state(PlayerState::Hurt);
        if self.player.health == 0 {
            self.die();
        }
    }
    fn die(&mut self) {
        self.phase = Phase::Dead;
        self.player.health = 0;
        self.player.vx = 0.0;
        self.set_state(PlayerState::Dead);
        self.events.push(Event::Death);
    }
    fn activate(&mut self, room: u8, index: usize) {
        let Some(switch) = self
            .level
            .room(room)
            .switches
            .iter()
            .find(|s| s.tile == index)
        else {
            return;
        };
        let opens = switch.opens;
        let targets = switch.targets.clone();
        let first = !self.runtime(room, index).pressed;
        self.tiles[usize::from(room) - 1][index].pressed = true;
        for target in targets {
            let rt = &mut self.tiles[usize::from(target.room) - 1][target.tile];
            rt.gate_ticks = if opens { 600 } else { 0 };
        }
        if first {
            self.events.push(Event::Gate);
        }
    }
    /// Advances exactly one simulation tick. Call at 60 Hz, independently of rendering.
    pub fn step(&mut self, input: Input) {
        self.events.clear();
        if self.phase != Phase::Playing {
            return;
        }
        self.tick = self.tick.wrapping_add(1);
        self.elapsed_ticks = self.elapsed_ticks.saturating_add(1);
        self.player.state_ticks = self.player.state_ticks.saturating_add(1);
        self.player.invulnerable = self.player.invulnerable.saturating_sub(1);
        self.player.landing_ticks = self.player.landing_ticks.saturating_sub(1);
        self.combat_feedback_ticks = self.combat_feedback_ticks.saturating_sub(1);
        self.update_tiles();
        let up = input.up && !self.previous.up;
        let action = input.action && !self.previous.action;
        self.attack_buffer = if action {
            10
        } else {
            self.attack_buffer.saturating_sub(1)
        };
        self.player.jump_buffer = if up {
            8
        } else {
            self.player.jump_buffer.saturating_sub(1)
        };
        self.player.coyote_ticks = if self.player.grounded {
            6
        } else {
            self.player.coyote_ticks.saturating_sub(1)
        };
        self.previous = input;
        if self.player.grounded && input.up {
            if let Some(index) = self.floor_index() {
                let room = self.player.room;
                let exit = match self.tile(room, index) {
                    Tile::Exit => Some(index),
                    Tile::ExitRight => Some(index.saturating_sub(1)),
                    _ => None,
                };
                if exit.is_some_and(|tile| self.runtime(room, tile).gate_open > 0.9) {
                    self.phase = Phase::Complete;
                    self.events.push(Event::Complete);
                    return;
                }
            }
        }
        if self.player.state == PlayerState::Hanging {
            self.player.vx = 0.0;
            self.player.vy = 0.0;
            if input.up {
                self.player.climb_start_x = self.player.x;
                self.player.climb_start_y = self.player.y;
                self.set_state(PlayerState::Climbing);
            } else if input.down || !input.action {
                self.set_state(PlayerState::Falling);
                self.player.y += 3.0;
                self.player.fall_distance = 0.0;
            }
            return;
        }
        if self.player.state == PlayerState::Climbing {
            let t = (f32::from(self.player.state_ticks) / 24.0).clamp(0.0, 1.0);
            let eased = t * t * (3.0 - 2.0 * t);
            let destination_x = self.player.hang_x
                + if self.player.hang_room == self.player.room {
                    0.0
                } else if self.player.hang_x > self.player.climb_start_x {
                    -ROOM_W
                } else {
                    ROOM_W
                };
            self.player.x =
                self.player.climb_start_x + (destination_x - self.player.climb_start_x) * eased;
            self.player.y = self.player.climb_start_y
                + (floor_y(self.player.hang_row) - self.player.climb_start_y) * eased;
            self.player.vx = 0.0;
            self.player.vy = 0.0;
            if self.player.state_ticks >= 24 {
                self.player.y = floor_y(self.player.hang_row);
                self.player.x = self.player.hang_x;
                self.player.room = self.player.hang_room;
                self.player.grounded = true;
                self.player.fall_distance = 0.0;
                self.set_state(PlayerState::Idle);
            }
            return;
        }
        let opponent = self
            .guards
            .iter()
            .filter(|g| {
                g.health > 0
                    && g.room == self.player.room
                    && (g.y - self.player.y).abs() < 15.0
                    && (g.x - self.player.x).abs() < 110.0
            })
            .min_by(|a, b| {
                (a.x - self.player.x)
                    .abs()
                    .total_cmp(&(b.x - self.player.x).abs())
            })
            .map(|g| g.x);
        let combat = opponent.is_some() && self.player.sword && self.player.grounded;
        let attacking = self.player.state == PlayerState::Attacking && self.player.state_ticks < 24;
        let defending =
            self.player.sword && self.player.grounded && (input.parry || (combat && input.up));
        // Sword input is held/repeatable. A completed attack explicitly restarts its clock.
        // Damage and recovery cannot be cancelled by spamming attack or defence.
        if self.player.state != PlayerState::Hurt && self.player.grounded {
            if !attacking && defending {
                if let Some(x) = opponent {
                    self.player.facing = if x < self.player.x { -1 } else { 1 };
                }
                self.set_state(PlayerState::Parrying);
                self.player.vx = 0.0;
            } else if !attacking
                && self.player.sword
                && (input.action || self.attack_buffer > 0)
                && !input.up
                && !input.down
                && (combat || (!input.left && !input.right))
            {
                if let Some(x) = opponent {
                    self.player.facing = if x < self.player.x { -1 } else { 1 };
                }
                self.set_state(PlayerState::Attacking);
                self.player.state_ticks = 0;
                self.player.vx = 0.0;
                self.attack_hit = false;
                self.attack_buffer = 0;
            } else if !attacking {
                let direction = i8::from(input.right) - i8::from(input.left);
                if combat {
                    if let Some(x) = opponent {
                        self.player.facing = if x < self.player.x { -1 } else { 1 };
                    }
                    self.player.vx = approach(self.player.vx, f32::from(direction) * 42.0, 9.0);
                    self.set_state(PlayerState::Combat);
                } else if direction != 0 {
                    if direction != self.player.facing {
                        if self.player.vx.abs() > 18.0 {
                            self.player.vx = approach(self.player.vx, 0.0, 18.0);
                            self.set_state(PlayerState::Stopping);
                        } else {
                            self.player.facing = direction;
                            self.set_state(PlayerState::Turning);
                            self.player.vx = 0.0;
                        }
                    } else if self.player.state != PlayerState::Turning
                        || self.player.state_ticks >= 4
                    {
                        let speed = if input.action { 32.0 } else { 100.0 };
                        self.player.vx =
                            approach(self.player.vx, f32::from(direction) * speed, 9.0);
                        self.set_state(if input.action {
                            PlayerState::Stepping
                        } else {
                            PlayerState::Running
                        });
                    }
                } else {
                    self.player.vx = approach(self.player.vx, 0.0, 12.0);
                    self.set_state(if input.down {
                        PlayerState::Crouching
                    } else if self.player.vx.abs() > 1.0 {
                        PlayerState::Stopping
                    } else {
                        PlayerState::Idle
                    });
                }
            }
        }
        if self.player.state == PlayerState::Hurt && self.player.state_ticks >= 15 {
            self.set_state(PlayerState::Idle);
        }
        if combat || defending {
            self.player.jump_buffer = 0;
        }
        let can_jump = self.player.grounded || self.player.coyote_ticks > 0;
        let climbing =
            self.player.jump_buffer > 0 && self.player.grounded && !combat && self.try_climb();
        if climbing {
            self.player.jump_buffer = 0;
        }
        if self.player.jump_buffer > 0
            && !combat
            && !defending
            && !attacking
            && can_jump
            && !climbing
        {
            self.player.jump_buffer = 0;
            self.player.coyote_ticks = 0;
            self.player.vy = -225.0;
            self.player.grounded = false;
            self.player.fall_distance = 0.0;
            if self.player.vx.abs() > 35.0 {
                self.player.vx = f32::from(self.player.facing) * 125.0;
            }
            self.set_state(PlayerState::Jumping);
            self.events.push(Event::Jump);
        }
        if self.player.state == PlayerState::Climbing {
            return;
        }
        if !self.player.grounded && input.left != input.right {
            let direction = if input.right { 1.0 } else { -1.0 };
            // A small steering force permits correction without erasing takeoff momentum.
            if self.player.vx * direction < 0.0 {
                self.player.vx = approach(self.player.vx, direction * 90.0, 1.6);
            }
        }
        if input.down {
            self.player.vx = approach(self.player.vx, 0.0, 20.0);
        }
        let old_x = self.player.x;
        let next_x = old_x + self.player.vx * DT;
        let edge = next_x + self.player.vx.signum() * HALF_WIDTH;
        if self.blocked(self.player.room, edge, self.player.y) {
            self.player.vx = 0.0;
        } else if self.player.grounded
            && input.action
            && !combat
            && !self.support(
                self.player.room,
                next_x + f32::from(self.player.facing) * if input.down { 12.0 } else { 5.0 },
                self.player.y,
            )
        {
            self.player.vx = 0.0;
            if input.down {
                self.player.grounded = false;
                self.player.x += f32::from(self.player.facing) * 9.0;
                self.player.y += 2.0;
            }
        } else {
            self.player.x = next_x;
        }
        if self.player.grounded && !self.support(self.player.room, self.player.x, self.player.y) {
            self.player.grounded = false;
            self.player.fall_distance = 0.0;
        }
        if !self.player.grounded {
            self.apply_gravity(input.action, input.up);
        }
        self.cross_rooms();
        if self.phase != Phase::Playing {
            return;
        }
        self.interact(input, action);
        self.update_guards();
        if self.player.grounded && self.phase == Phase::Playing {
            if let Some(index) = self.floor_index() {
                if Self::safe_tile(self.tile(self.player.room, index)) {
                    self.safe_floor = Some((self.player.room, index));
                }
            }
        }
        if self.elapsed_ticks >= 60 * 60 * 60 {
            self.die();
        }
    }
    fn update_tiles(&mut self) {
        for room in &mut self.tiles {
            for rt in room {
                rt.pressed = false;
                rt.gate_ticks = rt.gate_ticks.saturating_sub(1);
                rt.gate_open = approach(
                    rt.gate_open,
                    if rt.gate_ticks > 0 { 1.0 } else { 0.0 },
                    if rt.gate_ticks > 0 { 0.035 } else { 0.003 },
                );
                if rt.loose_ticks > 0 {
                    rt.loose_ticks -= 1;
                    if rt.loose_ticks == 0 {
                        rt.removed = true;
                    }
                }
            }
        }
    }
    fn try_climb(&mut self) -> bool {
        let p = &self.player;
        for row in 0..3 {
            let dy = p.y - floor_y(row);
            if (40.0..70.0).contains(&dy) {
                let ahead = p.x + f32::from(p.facing) * 20.0;
                if self.support(p.room, ahead, floor_y(row))
                    && !self.blocked(p.room, ahead, floor_y(row))
                {
                    let destination = self.horizontal_point(p.room, ahead);
                    self.player.hang_row = row;
                    self.player.climb_start_x = self.player.x;
                    self.player.climb_start_y = self.player.y;
                    if let Some((room, x)) = destination {
                        self.player.hang_room = room;
                        self.player.hang_x = x.clamp(8.0, ROOM_W - 8.0);
                    }
                    self.player.grounded = false;
                    self.set_state(PlayerState::Climbing);
                    return true;
                }
            }
        }
        false
    }
    fn apply_gravity(&mut self, grab: bool, hold_jump: bool) {
        let before = self.player.y;
        let gravity = if hold_jump && self.player.vy < 0.0 {
            480.0
        } else {
            600.0
        };
        self.player.vy = (self.player.vy + gravity * DT).min(420.0);
        self.player.y += self.player.vy * DT;
        if self.player.vy > 0.0 {
            self.player.fall_distance += self.player.vy * DT;
            self.set_state(PlayerState::Falling);
        }
        if self.player.vy < 0.0 {
            let head = self.player.y - HEIGHT;
            let col = (self.player.x / TILE_W).floor() as i32;
            if (0..10).contains(&col) {
                for row in 0..3 {
                    let idx = row * COLS + col as usize;
                    if self.tile(self.player.room, idx).supports()
                        && before - HEIGHT >= floor_y(row)
                        && head < floor_y(row)
                    {
                        self.player.y = floor_y(row) + HEIGHT;
                        self.player.vy = 0.0;
                        if self.tile(self.player.room, idx) == Tile::Loose {
                            self.start_loose(idx);
                        }
                        break;
                    }
                }
            }
        } else {
            let col = (self.player.x / TILE_W).floor() as i32;
            if (0..10).contains(&col) {
                for row in 0..3 {
                    let y = floor_y(row);
                    let index = row * COLS + col as usize;
                    if before <= y + 0.1
                        && self.player.y >= y
                        && self.tile(self.player.room, index).supports()
                    {
                        self.player.y = y;
                        self.player.vy = 0.0;
                        self.player.grounded = true;
                        self.player.landing_ticks = if self.player.fall_distance > 12.0 {
                            8
                        } else {
                            0
                        };
                        self.player.fall_distance = 0.0;
                        self.set_state(PlayerState::Idle);
                        self.events.push(Event::Land);
                        // Ordinary landings are safe at every height. Tile hazards
                        // remain responsible for damage after landing.
                        break;
                    }
                }
            }
            if grab && !self.player.grounded {
                for row in 0..3 {
                    let ledge_y = floor_y(row);
                    let ahead = self.player.x + f32::from(self.player.facing) * 12.0;
                    if (self.player.y - HEIGHT - ledge_y).abs() < 8.0
                        && self.support(self.player.room, ahead, ledge_y)
                        && !self.support(self.player.room, self.player.x, ledge_y)
                        && !self.blocked(self.player.room, ahead, ledge_y)
                    {
                        self.player.y = ledge_y + HEIGHT;
                        self.player.hang_row = row;
                        if let Some((room, x)) = self.horizontal_point(self.player.room, ahead) {
                            self.player.hang_room = room;
                            self.player.hang_x = x.clamp(8.0, ROOM_W - 8.0);
                        }
                        self.player.vy = 0.0;
                        self.player.fall_distance = 0.0;
                        self.set_state(PlayerState::Hanging);
                        self.events.push(Event::Grab);
                        break;
                    }
                }
            }
        }
    }
    fn start_loose(&mut self, index: usize) {
        let rt = &mut self.tiles[usize::from(self.player.room) - 1][index];
        if !rt.removed && rt.loose_ticks == 0 {
            rt.loose_ticks = 38;
            self.events.push(Event::Crumble);
        }
    }
    fn safe_tile(tile: Tile) -> bool {
        tile.supports() && !matches!(tile, Tile::Spikes | Tile::Chomper | Tile::Loose | Tile::Wall | Tile::Gate)
    }
    fn recover_from_void(&mut self) {
        let destination = self.safe_floor.filter(|&(room, index)| Self::safe_tile(self.tile(room, index)))
            .or_else(|| self.level.rooms.iter().find_map(|room| {
                (0..30).find(|&index| Self::safe_tile(self.tile(room.id, index))).map(|index| (room.id, index))
            }));
        if let Some((room, index)) = destination {
            self.player.room = room;
            self.player.x = tile_center(index);
            self.player.y = floor_y(index / COLS);
            self.player.vx = 0.0;
            self.player.vy = 0.0;
            self.player.fall_distance = 0.0;
            self.player.grounded = true;
            self.player.landing_ticks = 8;
            self.set_state(PlayerState::Idle);
            self.events.push(Event::Land);
        }
    }
    fn cross_rooms(&mut self) {
        let direction = if self.player.x < 0.0 {
            Some(0)
        } else if self.player.x >= ROOM_W {
            Some(1)
        } else if self.player.y < 0.0 {
            Some(2)
        } else if self.player.y > ROOM_H + 10.0 {
            Some(3)
        } else {
            None
        };
        let Some(dir) = direction else {
            return;
        };
        let next = self.level.room(self.player.room).links[dir];
        if next == 0 {
            if dir == 3 {
                self.recover_from_void();
            } else {
                self.player.x = self.player.x.clamp(6.0, ROOM_W - 6.0);
                self.player.vx = 0.0;
                if dir == 2 {
                    self.player.y = 0.0;
                    self.player.vy = 0.0;
                }
            }
            return;
        }
        let new_x = match dir {
            0 => self.player.x + ROOM_W,
            1 => self.player.x - ROOM_W,
            _ => self.player.x,
        };
        let new_y = match dir {
            2 => self.player.y + ROOM_H,
            3 => self.player.y - ROOM_H,
            _ => self.player.y,
        };
        if dir < 2 && self.blocked(next, new_x, new_y) {
            self.player.x = self.player.x.clamp(1.0, ROOM_W - 1.0);
            self.player.vx = 0.0;
            return;
        }
        self.player.room = next;
        self.player.x = new_x;
        self.player.y = new_y;
    }
    fn interact(&mut self, input: Input, action: bool) {
        if !self.player.grounded {
            return;
        }
        let Some(index) = self.floor_index() else {
            return;
        };
        let room = self.player.room;
        match self.tile(room, index) {
            Tile::Opener | Tile::Closer => self.activate(room, index),
            Tile::Loose => self.start_loose(index),
            Tile::Spikes => {
                if !input.action && self.player.invulnerable == 0 {
                    self.hurt(3);
                }
            }
            Tile::Sword if action && !self.player.sword => {
                self.tiles[usize::from(room) - 1][index].removed = false;
                self.player.sword = true;
                self.events.push(Event::Sword);
            }
            Tile::Potion if action && !self.runtime(room, index).consumed => {
                self.player.health = (self.player.health + 1).min(3);
                self.tiles[usize::from(room) - 1][index].consumed = true;
                self.events.push(Event::Drink);
            }
            Tile::Exit | Tile::ExitRight if input.up => {
                let exit = if self.tile(room, index) == Tile::ExitRight {
                    index.saturating_sub(1)
                } else {
                    index
                };
                if self.runtime(room, exit).gate_open > 0.9 {
                    self.phase = Phase::Complete;
                    self.events.push(Event::Complete);
                }
            }
            _ => {}
        }
        if self.player.state == PlayerState::Running && self.player.state_ticks % 14 == 0 {
            self.events.push(Event::Step);
        }
    }
    fn update_guards(&mut self) {
        let p = &self.player;
        let mut damage = false;
        let mut feedback = None;
        // Check the complete blade segment before taking a mutable guard borrow.
        // This avoids allocations and prevents hits across walls or narrow gates.
        for index in 0..self.guards.len() {
            let guard = &self.guards[index];
            if guard.health == 0 || guard.room != p.room || (guard.y - p.y).abs() > 16.0 {
                continue;
            }
            let steps = ((guard.x - p.x).abs() / 4.0).ceil().max(1.0) as u16;
            let clear = (0..=steps).all(|i| {
                let x = p.x + (guard.x - p.x) * f32::from(i) / f32::from(steps);
                !self.blocked(p.room, x, p.y)
            });
            if !clear {
                continue;
            }
            let guard = &mut self.guards[index];
            guard.hurt = guard.hurt.saturating_sub(1);
            let distance = (guard.x - p.x).abs();
            guard.facing = if p.x < guard.x { -1 } else { 1 };
            guard.cooldown = guard.cooldown.saturating_sub(1);
            if p.sword
                && p.grounded
                && p.state == PlayerState::Attacking
                && (6..=12).contains(&p.state_ticks)
                && !self.attack_hit
                && distance < 56.0
                && f32::from(p.facing) * (guard.x - p.x) > 0.0
            {
                guard.health = guard.health.saturating_sub(1);
                guard.hurt = 20;
                guard.cooldown = 65;
                guard.windup = 0;
                self.attack_hit = true;
                feedback = Some(Event::Hit);
                // A struck/dead guard cannot finish a cancelled windup this tick.
                continue;
            }
            if guard.hurt > 0 {
                continue;
            }
            if guard.windup > 0 {
                guard.windup -= 1;
                if guard.windup == 0 {
                    guard.cooldown = 70;
                    if distance < 52.0 && p.grounded {
                        if p.sword
                            && p.state == PlayerState::Parrying
                            && f32::from(p.facing) * (guard.x - p.x) > 0.0
                        {
                            if p.state_ticks <= 12 {
                                guard.hurt = 36;
                                guard.cooldown = 90;
                                feedback = Some(Event::Parry);
                            } else {
                                feedback = Some(Event::Clash);
                            }
                        } else {
                            damage = true;
                        }
                    }
                }
            } else if guard.cooldown == 0 && distance < 52.0 {
                guard.windup = 26;
            } else if distance > 38.0 && distance < 155.0 {
                guard.x += f32::from(guard.facing) * 24.0 * DT;
            }
        }
        if let Some(event) = feedback {
            self.events.push(event);
            self.last_combat_event = event;
            self.combat_feedback_ticks = 24;
        }
        if damage {
            self.hurt(1);
        }
    }
}
fn approach(value: f32, target: f32, delta: f32) -> f32 {
    if value < target {
        (value + delta).min(target)
    } else {
        (value - delta).max(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn game() -> Game {
        Game::new(Level::opening().expect("embedded level"))
    }
    fn place(g: &mut Game, room: u8, tile: usize) {
        g.player.room = room;
        g.player.x = tile_center(tile);
        g.player.y = floor_y(tile / COLS);
        g.player.grounded = true;
        g.player.vx = 0.0;
        g.player.vy = 0.0;
        g.player.state = PlayerState::Idle;
        g.player.invulnerable = 0;
    }
    #[test]
    fn deterministic_replay() {
        let mut a = game();
        let mut b = game();
        for tick in 0..1200 {
            let input = Input {
                right: tick % 240 < 120,
                left: tick % 240 >= 120,
                up: tick % 83 == 0,
                action: tick % 100 > 70,
                ..Input::default()
            };
            a.step(input);
            b.step(input);
        }
        assert_eq!(a.player.x.to_bits(), b.player.x.to_bits());
        assert_eq!(a.player.y.to_bits(), b.player.y.to_bits());
        assert_eq!(a.player.health, b.player.health);
        assert_eq!(a.player.room, b.player.room);
    }
    #[test]
    fn loose_floor_falls_and_room_crossing_works() {
        let mut g = game();
        place(&mut g, 1, 26);
        for _ in 0..100 {
            g.step(Input::default());
        }
        assert!(g.runtime(1, 26).removed);
        assert_eq!(g.player.room, 2);
        assert!(g.player.grounded);
    }
    #[test]
    fn sword_pickup_requires_action() {
        let mut g = game();
        place(&mut g, 15, 22);
        g.step(Input::default());
        assert!(!g.player.sword);
        g.step(Input {
            action: true,
            ..Input::default()
        });
        assert!(g.player.sword);
    }
    #[test]
    fn switch_opens_linked_gate_and_closer_closes_it() {
        let mut g = game();
        place(&mut g, 5, 4);
        g.step(Input::default());
        assert_eq!(g.runtime(5, 9).gate_ticks, 600);
        place(&mut g, 5, 2);
        g.step(Input::default());
        assert_eq!(g.runtime(5, 9).gate_ticks, 0);
    }
    #[test]
    fn spikes_kill_running_but_allow_careful_steps() {
        let mut g = game();
        place(&mut g, 13, 5);
        g.step(Input {
            action: true,
            ..Input::default()
        });
        assert_eq!(g.player.health, 3);
        g.step(Input::default());
        assert_eq!(g.phase, Phase::Dead);
    }
    #[test]
    fn exit_requires_open_door() {
        let mut g = game();
        place(&mut g, 9, 13);
        g.step(Input {
            up: true,
            ..Input::default()
        });
        assert_eq!(g.phase, Phase::Playing);
        g.activate(9, 0);
        for _ in 0..40 {
            g.step(Input::default());
        }
        place(&mut g, 9, 13);
        g.step(Input {
            up: true,
            ..Input::default()
        });
        assert_eq!(g.phase, Phase::Complete);
    }
    #[test]
    fn attack_can_defeat_guard() {
        let mut g = game();
        let guard = g.guards[0].clone();
        place(&mut g, guard.room, 17);
        g.player.x = guard.x - 36.0;
        g.player.facing = 1;
        g.player.sword = true;
        for _ in 0..3 {
            g.previous = Input::default();
            g.step(Input {
                action: true,
                ..Input::default()
            });
            for _ in 0..25 {
                g.step(Input::default());
            }
        }
        assert_eq!(g.guards[0].health, 0);
    }
    fn duel() -> Game {
        let mut g = game();
        g.start_duel();
        g
    }
    #[test]
    fn held_attack_repeats_and_each_swing_hits_once() {
        let mut g = duel();
        let input = Input {
            action: true,
            ..Input::default()
        };
        for _ in 0..20 {
            g.step(input);
        }
        assert_eq!(g.guards[0].health, 2, "one hit per swing");
        for _ in 0..55 {
            g.step(input);
        }
        assert_eq!(g.guards[0].health, 0, "holding attack must repeat");
        let health = g.player.health;
        for _ in 0..180 {
            g.step(Input::default());
        }
        assert_eq!(g.player.health, health, "dead guards cannot attack");
    }
    #[test]
    fn tap_during_recovery_queues_next_swing() {
        let mut g = duel();
        g.step(Input {
            action: true,
            ..Input::default()
        });
        for _ in 0..18 {
            g.step(Input::default());
        }
        g.step(Input {
            action: true,
            ..Input::default()
        });
        for _ in 0..14 {
            g.step(Input::default());
        }
        assert_eq!(g.guards[0].health, 1);
    }
    #[test]
    fn timed_parry_staggers_guard_and_allows_counterattack() {
        let mut g = duel();
        g.guards[0].windup = 2;
        for _ in 0..2 {
            g.step(Input {
                parry: true,
                ..Input::default()
            });
        }
        assert_eq!(g.player.health, 3);
        assert_eq!(g.last_combat_event, Event::Parry);
        assert!(g.guards[0].hurt > 0);
        for _ in 0..10 {
            g.step(Input {
                action: true,
                ..Input::default()
            });
        }
        assert_eq!(g.guards[0].health, 2, "parry creates a counter window");
    }
    #[test]
    fn held_block_protects_but_is_not_a_perfect_parry() {
        let mut g = duel();
        g.guards[0].windup = 30;
        for _ in 0..30 {
            g.step(Input {
                parry: true,
                ..Input::default()
            });
        }
        assert_eq!(g.player.health, 3);
        assert_eq!(g.last_combat_event, Event::Clash);
        assert_eq!(g.guards[0].hurt, 0);
        assert!(g.player.grounded);
    }
    #[test]
    fn up_blocks_in_combat_and_retreat_keeps_enemy_in_front() {
        let mut g = duel();
        let start = g.player.x;
        for _ in 0..6 {
            g.step(Input {
                left: true,
                ..Input::default()
            });
        }
        assert!(g.player.x < start);
        assert_eq!(g.player.facing, 1);
        g.guards[0].windup = 1;
        g.step(Input {
            up: true,
            ..Input::default()
        });
        assert_eq!(g.player.health, 3);
        assert_eq!(g.player.state, PlayerState::Parrying);
        assert!(g.player.grounded);
    }
    #[test]
    fn unarmed_block_fails_and_hurt_cannot_be_cancelled_by_attack() {
        let mut g = duel();
        g.player.sword = false;
        g.guards[0].windup = 1;
        g.step(Input {
            parry: true,
            ..Input::default()
        });
        assert_eq!(g.player.health, 2);
        g.player.sword = true;
        g.step(Input {
            action: true,
            ..Input::default()
        });
        assert_eq!(g.player.state, PlayerState::Hurt);
        assert_eq!(g.guards[0].health, 3);
    }
    #[test]
    fn walls_stop_both_swords() {
        let mut g = duel();
        let column = ((g.player.x + g.guards[0].x) * 0.5 / TILE_W) as usize;
        let row = ((g.player.y + 3.0) / TILE_H).round() as usize - 1;
        g.level.rooms[usize::from(g.player.room) - 1].rows[row]
            .replace_range(column..column + 1, "#");
        g.guards[0].windup = 1;
        for _ in 0..14 {
            g.step(Input {
                action: true,
                ..Input::default()
            });
        }
        assert_eq!(g.guards[0].health, 3);
        assert_eq!(g.player.health, 3);
    }
    #[test]
    fn swing_hits_when_guard_enters_active_window_and_not_out_of_range() {
        let mut g = duel();
        g.guards[0].x = g.player.x + 70.0;
        g.step(Input {
            action: true,
            ..Input::default()
        });
        for _ in 0..8 {
            g.step(Input::default());
        }
        assert_eq!(g.guards[0].health, 3);
        g.guards[0].x = g.player.x + 50.0;
        g.step(Input::default());
        assert_eq!(g.guards[0].health, 2, "active window lasts past tick eight");
    }
    #[test]
    fn long_falls_on_plain_stone_do_not_damage_even_at_one_health() {
        let mut g = game();
        g.guards.clear();
        g.level.rooms[0].rows = ["          ".into(), "          ".into(), "..........".into()];
        g.player.room = 1;
        g.player.x = 16.0;
        g.player.y = 0.0;
        g.player.vy = 450.0;
        g.player.grounded = false;
        g.player.health = 1;
        g.player.fall_distance = 500.0;
        for _ in 0..60 { g.step(Input::default()); }
        assert_eq!(g.phase, Phase::Playing);
        assert_eq!(g.player.health, 1);
        assert!(g.player.grounded);
        assert_eq!(g.player.y, floor_y(2));
    }
    #[test]
    fn long_drop_between_rooms_is_safe_on_normal_floor() {
        let mut g = game();
        g.guards.clear();
        g.level.rooms[0].rows = ["          ".into(), "          ".into(), "          ".into()];
        g.level.rooms[0].links[3] = 2;
        g.level.rooms[1].rows = ["          ".into(), "          ".into(), "..........".into()];
        g.player.x = 16.0;
        g.player.y = 180.0;
        g.player.vy = 300.0;
        g.player.grounded = false;
        for _ in 0..90 { g.step(Input::default()); }
        assert_eq!(g.player.room, 2);
        assert_eq!(g.player.health, 3);
        assert_eq!(g.phase, Phase::Playing);
        assert!(g.player.grounded);
    }
    #[test]
    fn falling_onto_spikes_is_still_lethal() {
        let mut g = game();
        g.guards.clear();
        g.level.rooms[0].rows = ["          ".into(), "          ".into(), "s.........".into()];
        g.player.x = 16.0;
        g.player.y = 0.0;
        g.player.vy = 450.0;
        g.player.grounded = false;
        for _ in 0..60 { g.step(Input::default()); }
        assert_eq!(g.phase, Phase::Dead);
    }
    #[test]
    fn unmarked_bottom_gap_recovers_to_last_safe_floor_without_damage() {
        let mut g = game();
        g.guards.clear();
        place(&mut g, 3, 12);
        g.step(Input::default());
        let safe = g.safe_floor.expect("safe floor recorded");
        g.player.room = 1;
        g.level.rooms[0].links[3] = 0;
        g.player.y = ROOM_H + 11.0;
        g.player.grounded = false;
        g.cross_rooms();
        assert_eq!(g.player.room, safe.0);
        assert_eq!(g.player.x, tile_center(safe.1));
        assert_eq!(g.player.health, 3);
        assert_eq!(g.phase, Phase::Playing);
        assert!(g.player.grounded);
    }
    #[test]
    fn jump_returns_to_floor() {
        let mut g = game();
        place(&mut g, 3, 12);
        g.step(Input {
            up: true,
            ..Input::default()
        });
        assert_eq!(g.player.state, PlayerState::Jumping);
        for _ in 0..60 {
            g.step(Input::default());
        }
        assert!(g.player.grounded);
        assert!((g.player.y - floor_y(1)).abs() < 0.1);
    }
    #[test]
    fn climb_can_cross_a_room_boundary() {
        let mut g = game();
        place(&mut g, 2, 10);
        g.player.x = 9.0;
        g.player.facing = -1;
        g.step(Input {
            up: true,
            ..Input::default()
        });
        assert_eq!(g.player.state, PlayerState::Climbing);
        for _ in 0..26 {
            g.step(Input::default());
        }
        assert_eq!(g.player.room, 6);
        assert!(g.player.grounded);
        assert_eq!(g.player.y, floor_y(0));
    }
    #[test]
    fn potion_is_consumed_without_removing_the_floor() {
        let mut g = game();
        place(&mut g, 5, 23);
        g.player.health = 1;
        g.step(Input {
            action: true,
            ..Input::default()
        });
        assert_eq!(g.player.health, 2);
        g.step(Input::default());
        g.step(Input {
            action: true,
            ..Input::default()
        });
        assert_eq!(g.player.health, 2);
        assert!(g.tile(5, 23).supports());
    }
    #[test]
    fn jump_has_a_short_ledge_grace_period() {
        let mut g = game();
        place(&mut g, 8, 16);
        g.player.x = 162.0;
        g.player.facing = -1;
        g.player.vx = -100.0;
        g.step(Input {
            left: true,
            ..Input::default()
        });
        g.step(Input {
            left: true,
            ..Input::default()
        });
        assert!(!g.player.grounded);
        g.step(Input {
            up: true,
            left: true,
            ..Input::default()
        });
        assert!(g.player.vy < 0.0);
        assert_eq!(g.player.state, PlayerState::Jumping);
    }
    #[test]
    fn reversing_direction_preserves_deceleration() {
        let mut g = game();
        place(&mut g, 3, 13);
        g.player.facing = 1;
        g.player.vx = 100.0;
        g.step(Input {
            left: true,
            ..Input::default()
        });
        assert!(g.player.vx > 0.0 && g.player.vx < 100.0);
        assert_eq!(g.player.state, PlayerState::Stopping);
    }
    #[test]
    fn climbing_moves_continuously_before_arriving() {
        let mut g = game();
        place(&mut g, 2, 10);
        g.player.x = 9.0;
        g.player.facing = -1;
        g.step(Input {
            up: true,
            ..Input::default()
        });
        let start_y = g.player.y;
        for _ in 0..10 {
            g.step(Input::default());
        }
        assert!(g.player.y < start_y && g.player.y > 60.0);
        for _ in 0..15 {
            g.step(Input::default());
        }
        assert_eq!(g.player.room, 6);
        assert_eq!(g.player.y, 60.0);
    }
}
