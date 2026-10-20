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
    fall_distance: f32,
    hang_row: usize,
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
    previous: Input,
    attack_hit: bool,
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
                fall_distance: 0.0,
                hang_row: 0,
            },
            guards,
            tiles,
            phase: Phase::Playing,
            tick: 0,
            elapsed_ticks: 0,
            events: Vec::with_capacity(8),
            previous: Input::default(),
            attack_hit: false,
        }
    }
    pub fn restart(&mut self) {
        *self = Self::new(self.level.clone());
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
    fn support(&self, room: u8, x: f32, y: f32) -> bool {
        if !(0.0..ROOM_W).contains(&x) {
            return false;
        }
        (0..3).any(|r| {
            (floor_y(r) - y).abs() < 1.0
                && self.tile(room, r * COLS + (x / TILE_W) as usize).supports()
        })
    }
    fn blocked(&self, room: u8, x: f32, y: f32) -> bool {
        if !(0.0..ROOM_W).contains(&x) {
            return false;
        }
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
        self.update_tiles();
        let up = input.up && !self.previous.up;
        let action = input.action && !self.previous.action;
        self.previous = input;
        if self.player.state == PlayerState::Hanging {
            self.player.vx = 0.0;
            self.player.vy = 0.0;
            if input.up {
                self.set_state(PlayerState::Climbing);
            } else if input.down || !input.action {
                self.set_state(PlayerState::Falling);
                self.player.y += 3.0;
                self.player.fall_distance = 0.0;
            }
            return;
        }
        if self.player.state == PlayerState::Climbing {
            if self.player.state_ticks >= 18 {
                self.player.y = floor_y(self.player.hang_row);
                self.player.x += f32::from(self.player.facing) * 14.0;
                self.player.grounded = true;
                self.player.fall_distance = 0.0;
                self.set_state(PlayerState::Idle);
            }
            return;
        }
        let threat = self.guards.iter().any(|g| {
            g.health > 0
                && g.room == self.player.room
                && (g.y - self.player.y).abs() < 15.0
                && (g.x - self.player.x).abs() < 95.0
        });
        let combat = threat && self.player.sword && self.player.grounded;
        let attacking = self.player.state == PlayerState::Attacking && self.player.state_ticks < 22;
        if combat && action && !attacking {
            self.set_state(PlayerState::Attacking);
            self.attack_hit = false;
        } else if !attacking && self.player.state != PlayerState::Hurt {
            let direction = i8::from(input.right) - i8::from(input.left);
            if combat && input.up {
                self.set_state(PlayerState::Parrying);
                self.player.vx = 0.0;
            } else if direction != 0 {
                if direction != self.player.facing && self.player.grounded {
                    self.player.facing = direction;
                    self.set_state(PlayerState::Turning);
                    self.player.vx = 0.0;
                } else if self.player.state != PlayerState::Turning || self.player.state_ticks >= 7
                {
                    let speed = if combat {
                        42.0
                    } else if input.action {
                        32.0
                    } else {
                        100.0
                    };
                    let target = f32::from(direction) * speed;
                    self.player.vx = approach(self.player.vx, target, 9.0);
                    if self.player.grounded {
                        self.set_state(if combat {
                            PlayerState::Combat
                        } else if input.action {
                            PlayerState::Stepping
                        } else {
                            PlayerState::Running
                        });
                    }
                }
            } else {
                self.player.vx = approach(self.player.vx, 0.0, 12.0);
                if self.player.grounded {
                    self.set_state(if input.down {
                        PlayerState::Crouching
                    } else if combat {
                        PlayerState::Combat
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
        if up && !combat && self.player.grounded && !self.try_climb() {
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
                next_x + f32::from(self.player.facing) * 5.0,
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
            self.apply_gravity(input.action);
        }
        self.cross_rooms();
        if self.phase != Phase::Playing {
            return;
        }
        self.interact(input, action);
        self.update_guards();
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
                    self.player.hang_row = row;
                    self.player.grounded = false;
                    self.set_state(PlayerState::Climbing);
                    return true;
                }
            }
        }
        false
    }
    fn apply_gravity(&mut self, grab: bool) {
        let before = self.player.y;
        self.player.vy = (self.player.vy + 600.0 * DT).min(420.0);
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
                        let distance = self.player.fall_distance;
                        self.player.fall_distance = 0.0;
                        self.set_state(PlayerState::Idle);
                        self.events.push(Event::Land);
                        if distance > 145.0 {
                            self.hurt(3);
                        } else if distance > 85.0 {
                            self.hurt(1);
                        }
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
                self.die();
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
            Tile::Sword if action => {
                self.tiles[usize::from(room) - 1][index].removed = false;
                self.player.sword = true;
                self.events.push(Event::Sword);
            }
            Tile::Potion if action && !self.runtime(room, index).pressed => {
                self.player.health = (self.player.health + 1).min(3);
                self.tiles[usize::from(room) - 1][index].pressed = true;
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
        for guard in &mut self.guards {
            guard.hurt = guard.hurt.saturating_sub(1);
            if guard.health == 0 || guard.room != p.room || (guard.y - p.y).abs() > 16.0 {
                continue;
            }
            let distance = (guard.x - p.x).abs();
            guard.facing = if p.x < guard.x { -1 } else { 1 };
            guard.cooldown = guard.cooldown.saturating_sub(1);
            if p.state == PlayerState::Attacking
                && p.state_ticks == 8
                && !self.attack_hit
                && distance < 51.0
                && f32::from(p.facing) * (guard.x - p.x) > 0.0
            {
                guard.health = guard.health.saturating_sub(1);
                guard.hurt = 20;
                guard.cooldown = 65;
                guard.windup = 0;
                self.attack_hit = true;
                self.events.push(Event::Clash);
            }
            if guard.windup > 0 {
                guard.windup -= 1;
                if guard.windup == 0 && distance < 49.0 {
                    if p.state == PlayerState::Parrying
                        && f32::from(p.facing) * (guard.x - p.x) > 0.0
                    {
                        self.events.push(Event::Clash);
                    } else {
                        damage = true;
                    }
                    guard.cooldown = 70;
                }
            } else if guard.cooldown == 0 && distance < 48.0 {
                guard.windup = 26;
            } else if distance > 37.0 && distance < 155.0 && guard.hurt == 0 {
                guard.x += f32::from(guard.facing) * 24.0 * DT;
            }
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
}
