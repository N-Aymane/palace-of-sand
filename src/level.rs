//! Validated, renderer-independent dungeon geometry.
use serde::Deserialize;

pub const COLS: usize = 10;
pub const ROWS: usize = 3;
pub const TILE_W: f32 = 32.0;
pub const TILE_H: f32 = 63.0;
pub const ROOM_W: f32 = 320.0;
pub const ROOM_H: f32 = 189.0;

/// A tile's gameplay role. Decorative pillars and torches also support the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Floor,
    Wall,
    Spikes,
    Gate,
    Loose,
    Closer,
    Opener,
    Potion,
    Exit,
    ExitRight,
    Sword,
    Torch,
    Pillar,
    Debris,
    Skeleton,
    Chomper,
}
impl Tile {
    pub fn from_glyph(c: char) -> Option<Self> {
        Some(match c {
            ' ' => Self::Empty,
            '.' => Self::Floor,
            '#' => Self::Wall,
            's' => Self::Spikes,
            'g' => Self::Gate,
            'l' => Self::Loose,
            '-' => Self::Closer,
            '+' => Self::Opener,
            'p' => Self::Potion,
            'x' => Self::Exit,
            'e' => Self::ExitRight,
            'w' => Self::Sword,
            '*' => Self::Torch,
            'o' => Self::Pillar,
            'b' => Self::Debris,
            'k' => Self::Skeleton,
            'c' => Self::Chomper,
            _ => return None,
        })
    }
    pub fn supports(self) -> bool {
        self != Self::Empty
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Location {
    pub room: u8,
    pub tile: usize,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Switch {
    pub tile: usize,
    pub opens: bool,
    pub targets: Vec<Location>,
}
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct GuardSpawn {
    pub tile: usize,
    pub facing: i8,
}
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Start {
    pub room: u8,
    pub tile: usize,
    pub facing: i8,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Room {
    pub id: u8,
    /// Left, right, up, down. Zero is a sealed boundary.
    pub links: [u8; 4],
    pub rows: [String; ROWS],
    pub switches: Vec<Switch>,
    pub guard: Option<GuardSpawn>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Level {
    pub start: Start,
    pub rooms: Vec<Room>,
}
impl Level {
    /// Loads and validates the embedded opening level before gameplay begins.
    pub fn opening() -> Result<Self, String> {
        let level: Self = serde_json::from_str(include_str!("../assets/opening.json"))
            .map_err(|e| format!("invalid level JSON: {e}"))?;
        level.validate()?;
        Ok(level)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.rooms.is_empty() || self.rooms.len() > 24 {
            return Err("invalid room count".into());
        }
        for (index, room) in self.rooms.iter().enumerate() {
            if usize::from(room.id) != index + 1 {
                return Err("room IDs must be consecutive".into());
            }
            for row in &room.rows {
                if row.chars().count() != COLS || row.chars().any(|c| Tile::from_glyph(c).is_none())
                {
                    return Err(format!("invalid tiles in room {}", room.id));
                }
            }
            if room
                .links
                .iter()
                .any(|&id| usize::from(id) > self.rooms.len())
            {
                return Err("invalid room link".into());
            }
            if room
                .guard
                .is_some_and(|g| g.tile >= 30 || ![-1, 1].contains(&g.facing))
            {
                return Err("invalid guard".into());
            }
            for switch in &room.switches {
                if switch.tile >= 30
                    || !matches!(self.tile(room.id, switch.tile), Tile::Opener | Tile::Closer)
                {
                    return Err("invalid switch".into());
                }
                for target in &switch.targets {
                    if target.room == 0
                        || usize::from(target.room) > self.rooms.len()
                        || target.tile >= 30
                    {
                        return Err("invalid switch target".into());
                    }
                    if !matches!(self.tile(target.room, target.tile), Tile::Gate | Tile::Exit) {
                        return Err("switch must target a gate or exit".into());
                    }
                }
            }
        }
        if self.start.room == 0
            || usize::from(self.start.room) > self.rooms.len()
            || self.start.tile >= 30
            || ![-1, 1].contains(&self.start.facing)
        {
            return Err("invalid start".into());
        }
        Ok(())
    }
    pub fn room(&self, id: u8) -> &Room {
        &self.rooms[usize::from(id) - 1]
    }
    pub fn tile(&self, room: u8, index: usize) -> Tile {
        self.room(room)
            .rows
            .get(index / COLS)
            .and_then(|row| row.chars().nth(index % COLS))
            .and_then(Tile::from_glyph)
            .unwrap_or(Tile::Wall)
    }
}
pub fn floor_y(row: usize) -> f32 {
    (row as f32 + 1.0) * TILE_H - 3.0
}
pub fn tile_center(index: usize) -> f32 {
    (index % COLS) as f32 * TILE_W + TILE_W / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_geometry_matches_reference() {
        let level = Level::opening().expect("embedded level");
        assert_eq!(level.rooms.len(), 24);
        assert_eq!(level.room(1).links, [5, 0, 0, 2]);
        assert_eq!(level.tile(15, 22), Tile::Sword);
        assert_eq!(level.tile(9, 13), Tile::Exit);
        assert_eq!(level.rooms.iter().filter(|r| r.guard.is_some()).count(), 2);
    }
    #[test]
    fn malformed_data_is_rejected() {
        let mut level = Level::opening().expect("embedded level");
        level.rooms[0].links[0] = 25;
        assert!(level.validate().is_err());
    }
}
