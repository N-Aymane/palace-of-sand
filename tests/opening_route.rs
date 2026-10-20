//! A real input-only playthrough: no teleporting, invulnerability, or geometry edits.
use palace_of_sand::level::Level;
use palace_of_sand::{Game, Input, Phase, PlayerState};

#[track_caller]
fn advance(g: &mut Game, input: Input, ticks: usize) {
    for _ in 0..ticks {
        g.step(input);
        assert_ne!(g.phase, Phase::Dead, "died: {:?}", g.player);
    }
}
#[track_caller]
fn until(g: &mut Game, input: Input, goal: impl Fn(&Game) -> bool) {
    for _ in 0..900 {
        if goal(g) {
            return;
        }
        advance(g, input, 1);
    }
    panic!("waypoint timed out: {:?}", g.player);
}
fn left() -> Input {
    Input {
        left: true,
        ..Input::default()
    }
}
fn right() -> Input {
    Input {
        right: true,
        ..Input::default()
    }
}
fn settle(g: &mut Game) {
    advance(g, Input::default(), 25);
}
fn jump(g: &mut Game, dir: Input) {
    advance(g, Input { up: true, ..dir }, 1);
}
fn climb(g: &mut Game) {
    advance(
        g,
        Input {
            up: true,
            ..Input::default()
        },
        20,
    );
    advance(g, Input::default(), 1);
}

#[test]
fn opening_level_can_be_completed_using_only_controls() {
    let mut g = Game::new(Level::opening().expect("level"));
    settle(&mut g);
    // Cell → lower corridor → ascent into the western dungeon.
    until(&mut g, right(), |g| g.player.x >= 207.0);
    settle(&mut g);
    until(&mut g, Input::default(), |g| {
        g.player.room == 2 && g.player.grounded
    });
    until(&mut g, left(), |g| g.player.x < 10.0);
    climb(&mut g);
    assert_eq!(g.player.room, 6);
    until(&mut g, left(), |g| g.player.x < 146.0);
    jump(&mut g, left());
    until(&mut g, Input::default(), |g| g.player.grounded);
    settle(&mut g);
    until(&mut g, right(), |g| g.player.x > 80.0);
    settle(&mut g);
    until(&mut g, left(), |g| g.player.room == 8 && g.player.x < 245.0);
    settle(&mut g);
    until(
        &mut g,
        Input {
            left: true,
            action: true,
            ..Input::default()
        },
        |g| g.player.x < 200.0,
    );
    advance(
        &mut g,
        Input {
            left: true,
            down: true,
            action: true,
            ..Input::default()
        },
        1,
    );
    until(&mut g, Input::default(), |g| {
        g.player.grounded && g.player.y > 120.0
    });
    settle(&mut g);
    until(&mut g, right(), |g| g.player.x > 240.0);
    settle(&mut g);
    until(&mut g, left(), |g| g.player.x < 176.0);
    jump(&mut g, left());
    until(&mut g, left(), |g| g.player.x < 10.0 && g.player.grounded);
    climb(&mut g);
    assert_eq!(g.player.room, 7);
    until(&mut g, left(), |g| {
        g.player.room == 20 && g.player.x < 195.0
    });
    jump(&mut g, left());
    until(&mut g, left(), |g| g.player.x < 10.0 && g.player.grounded);
    climb(&mut g);
    assert_eq!(g.player.room, 12);
    until(&mut g, left(), |g| g.player.x < 209.0);
    jump(&mut g, left());
    until(&mut g, left(), |g| {
        g.player.room == 15 && g.player.x < 160.0
    });
    until(&mut g, Input::default(), |g| {
        g.player.grounded && g.player.y > 180.0
    });
    settle(&mut g);
    // Stop over the sword, pick it up, and climb back to the return route.
    if g.player.x > 86.0 {
        until(&mut g, left(), |g| g.player.x < 83.0);
    } else {
        until(&mut g, right(), |g| g.player.x > 77.0);
    }
    settle(&mut g);
    advance(
        &mut g,
        Input {
            action: true,
            ..Input::default()
        },
        1,
    );
    assert!(g.player.sword, "{:?}", g.player);
    until(&mut g, right(), |g| g.player.x > 145.0);
    climb(&mut g);
    climb(&mut g);
    assert_eq!(g.player.y, 60.0);
    until(&mut g, right(), |g| {
        g.player.room == 12 && g.player.x > 15.0
    });
    jump(&mut g, right());
    until(&mut g, right(), |g| g.player.x > 111.0 && g.player.grounded);
    jump(&mut g, right());
    until(&mut g, right(), |g| {
        g.player.room == 20 && g.player.x > 108.0 && g.player.grounded
    });
    jump(&mut g, right());
    until(&mut g, right(), |g| {
        g.player.room == 7 && g.player.x > 50.0 && g.player.grounded
    });
    climb(&mut g);
    until(&mut g, right(), |g| g.player.x > 143.0);
    jump(&mut g, right());
    until(&mut g, right(), |g| {
        g.player.room == 8 && g.player.x > 108.0 && g.player.grounded
    });
    jump(&mut g, right());
    until(&mut g, Input::default(), |g| g.player.grounded);
    climb(&mut g);
    until(&mut g, right(), |g| g.player.room == 6 && g.player.x > 80.0);
    jump(&mut g, right());
    until(&mut g, right(), |g| g.player.room == 3 && g.player.grounded);
    // Approach the guard and repeatedly strike during the recovery window.
    until(&mut g, right(), |g| {
        g.guards.iter().any(|guard| {
            guard.room == g.player.room && guard.health > 0 && (guard.x - g.player.x).abs() < 45.0
        })
    });
    for _ in 0..3 {
        advance(
            &mut g,
            Input {
                action: true,
                ..Input::default()
            },
            1,
        );
        advance(&mut g, Input::default(), 25);
    }
    assert_eq!(g.guards[0].health, 0);
    until(&mut g, right(), |g| g.player.room == 9 && g.player.x > 43.0);
    settle(&mut g);
    advance(&mut g, left(), 1);
    climb(&mut g);
    assert_eq!(g.player.y, 60.0);
    advance(&mut g, Input::default(), 2);
    until(&mut g, right(), |g| g.player.x > 113.0 && g.player.grounded);
    settle(&mut g);
    advance(
        &mut g,
        Input {
            up: true,
            ..Input::default()
        },
        1,
    );
    assert_eq!(g.phase, Phase::Complete, "{:?}", g.player);
    assert!(g.elapsed_ticks < 60 * 60 * 60);
    assert_ne!(g.player.state, PlayerState::Dead);
}
