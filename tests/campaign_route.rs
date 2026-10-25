//! Input-only routes through every new chapter, including combat and switches.
use palace_of_sand::level::{Level, Tile};
use palace_of_sand::{Game, Input, Phase};

#[test]
fn all_new_chapters_can_be_completed_with_controls() {
    for chapter in 1..4 {
        let mut game = Game::new(Level::campaign(chapter).expect("campaign"));
        for _ in 0..18000 {
            if game.phase == Phase::Complete {
                break;
            }
            assert_ne!(
                game.phase,
                Phase::Dead,
                "chapter {chapter}: {:?}",
                game.player
            );
            let guard = game.guards.iter().find(|guard| {
                game.player.grounded
                    && guard.room == game.player.room
                    && guard.health > 0
                    && (guard.x - game.player.x).abs() < 105.0
            });
            let jump_hazard = game.player.grounded
                && (1..=40).any(|offset| {
                    let column = ((game.player.x + offset as f32) / 32.0) as usize;
                    column < 10
                        && matches!(
                            game.tile(game.player.room, 20 + column),
                            Tile::Spikes | Tile::Chomper
                        )
                });
            let input = if jump_hazard && guard.is_none() {
                Input {
                    right: true,
                    up: true,
                    ..Input::default()
                }
            } else if let Some(guard) = guard {
                if (guard.x - game.player.x).abs() < 50.0 {
                    Input {
                        action: true,
                        ..Input::default()
                    }
                } else {
                    Input {
                        right: true,
                        ..Input::default()
                    }
                }
            } else {
                let tile = game
                    .floor_index()
                    .map(|index| game.tile(game.player.room, index));
                Input {
                    right: true,
                    action: !matches!(tile, Some(Tile::Exit | Tile::ExitRight)),
                    up: matches!(tile, Some(Tile::Exit | Tile::ExitRight)),
                    ..Input::default()
                }
            };
            game.step(input);
        }
        assert_eq!(
            game.phase,
            Phase::Complete,
            "chapter {chapter}: {:?}",
            game.player
        );
        assert!(game.guards.iter().all(|guard| guard.health == 0));
    }
}
