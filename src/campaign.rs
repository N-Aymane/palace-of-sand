//! Campaign progression and validated, versioned shrine checkpoints.
use crate::level::Level;
use crate::{Game, Guard, Phase, Player, PlayerState, TileRuntime};
use serde::{Deserialize, Serialize};

pub const TITLES: [&str; 4] = [
    "I / THE DUNGEON",
    "II / COURTYARD",
    "III / CISTERN",
    "IV / THE PALACE",
];

/// Checkpoints contain mutable simulation state, never executable code or geometry.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    version: u8,
    pub chapter: usize,
    player: Player,
    guards: Vec<Guard>,
    tiles: Vec<[TileRuntime; 30]>,
    tick: u64,
    elapsed_ticks: u32,
}

impl Checkpoint {
    pub fn capture(chapter: usize, game: &Game) -> Result<String, String> {
        if chapter >= TITLES.len() || !game.at_save_point() {
            return Err("Stand at a SAVE shrine, away from guards.".into());
        }
        serde_json::to_string(&Self {
            version: 1,
            chapter,
            player: game.player.clone(),
            guards: game.guards.clone(),
            tiles: game.tiles.clone(),
            tick: game.tick,
            elapsed_ticks: game.elapsed_ticks,
        })
        .map_err(|error| error.to_string())
    }

    pub fn restore(source: &str) -> Result<(usize, Game), String> {
        if source.len() > 150_000 {
            return Err("Checkpoint is too large.".into());
        }
        let saved: Self =
            serde_json::from_str(source).map_err(|_| "Invalid checkpoint.".to_string())?;
        if saved.version != 1 || saved.chapter >= TITLES.len() || saved.elapsed_ticks >= 216_000 {
            return Err("Unsupported or expired checkpoint.".into());
        }
        let mut game = Game::new(Level::campaign(saved.chapter)?);
        let p = &saved.player;
        if p.room == 0
            || usize::from(p.room) > game.level.rooms.len()
            || !p.x.is_finite()
            || !p.y.is_finite()
            || !(0.0..320.0).contains(&p.x)
            || !(0.0..189.0).contains(&p.y)
            || !(1..=3).contains(&p.health)
            || ![-1, 1].contains(&p.facing)
            || saved.tiles.len() != game.tiles.len()
            || saved.guards.len() != game.guards.len()
        {
            return Err("Invalid checkpoint state.".into());
        }
        for (guard, original) in saved.guards.iter().zip(&game.guards) {
            if guard.room != original.room
                || !guard.x.is_finite()
                || !guard.y.is_finite()
                || !(0.0..320.0).contains(&guard.x)
                || guard.y != original.y
                || guard.health > 3
                || ![-1, 1].contains(&guard.facing)
            {
                return Err("Invalid checkpoint guard.".into());
            }
        }
        if saved
            .tiles
            .iter()
            .flatten()
            .any(|tile| !tile.gate_open.is_finite() || !(0.0..=1.0).contains(&tile.gate_open))
        {
            return Err("Invalid checkpoint tiles.".into());
        }
        // Normalize transient movement/input so loading cannot resume a fall or held attack.
        game.player.room = p.room;
        game.player.x = p.x;
        game.player.y = p.y;
        game.player.facing = p.facing;
        game.player.health = p.health;
        game.player.sword = p.sword;
        game.player.grounded = true;
        game.player.state = PlayerState::Idle;
        game.guards = saved.guards;
        game.tiles = saved.tiles;
        game.tick = saved.tick;
        game.elapsed_ticks = saved.elapsed_ticks;
        game.phase = Phase::Playing;
        if !game.at_save_point() {
            return Err("Checkpoint is outside a safe shrine.".into());
        }
        let index = game.floor_index().ok_or("Checkpoint has no floor.")?;
        if !game.tile(p.room, index).supports() {
            return Err("Checkpoint floor is missing.".into());
        }
        Ok((saved.chapter, game))
    }
}

/// Completing a chapter carries the campaign clock into the next one.
pub fn advance(chapter: usize, game: &Game) -> Result<Option<(usize, Game)>, String> {
    if game.phase != Phase::Complete {
        return Err("The exit has not been reached.".into());
    }
    if chapter + 1 >= TITLES.len() {
        return Ok(None);
    }
    let next = chapter + 1;
    let mut game_next = Game::new(Level::campaign(next)?);
    game_next.elapsed_ticks = game.elapsed_ticks;
    Ok(Some((next, game_next)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn campaign_levels_validate_and_progress_without_resetting_time() {
        for chapter in 0..TITLES.len() {
            let mut game = Game::new(Level::campaign(chapter).expect("campaign"));
            game.phase = Phase::Complete;
            game.elapsed_ticks = 2345;
            let next = advance(chapter, &game).expect("advance");
            if chapter == TITLES.len() - 1 {
                assert!(next.is_none());
            } else {
                let (index, next) = next.expect("next");
                assert_eq!(index, chapter + 1);
                assert_eq!(next.elapsed_ticks, 2345);
                assert!(next.player.sword);
            }
        }
    }
    #[test]
    fn checkpoint_roundtrip_preserves_progress_and_rejects_bad_state() {
        let mut game = Game::new(Level::campaign(1).expect("chapter"));
        for _ in 0..2 {
            game.step(crate::Input::default());
        }
        assert!(game.at_save_point());
        game.player.health = 2;
        game.tiles[1][22].gate_open = 0.75;
        game.guards[0].health = 1;
        let source = Checkpoint::capture(1, &game).expect("save");
        let (chapter, restored) = Checkpoint::restore(&source).expect("load");
        assert_eq!(chapter, 1);
        assert_eq!(restored.player.health, 2);
        assert_eq!(restored.guards[0].health, 1);
        assert_eq!(restored.tiles[1][22].gate_open, 0.75);
        let mut data: serde_json::Value = serde_json::from_str(&source).expect("json");
        data["player"]["room"] = 255.into();
        assert!(Checkpoint::restore(&data.to_string()).is_err());
        game.player.x = 250.0;
        assert!(Checkpoint::capture(1, &game).is_err());
    }
}
