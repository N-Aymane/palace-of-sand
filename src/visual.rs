//! High-resolution raster art and presentation-only animation.
//! Simulation coordinates remain 320×189; artwork never changes collisions.
use macroquad::prelude::*;
use palace_of_sand::level::{COLS, Tile, floor_y};
use palace_of_sand::{Event, Game, Player, PlayerState};

pub struct Art {
    backdrop: Texture2D,
    stone: Texture2D,
    props: Texture2D,
    run: Texture2D,
    actions: Texture2D,
    combat: Texture2D,
}
impl Art {
    pub async fn load() -> Result<Self, String> {
        async fn texture(name: &str) -> Result<Texture2D, String> {
            let texture = load_texture(&format!("/art/{name}.png"))
                .await
                .map_err(|e| format!("could not load {name}: {e}"))?;
            texture.set_filter(FilterMode::Linear);
            Ok(texture)
        }
        Ok(Self {
            backdrop: texture("dungeon-backdrop").await?,
            stone: texture("stone-wall").await?,
            props: texture("dungeon-props").await?,
            run: texture("hero-run").await?,
            actions: texture("hero-actions").await?,
            combat: texture("hero-combat-v2").await?,
        })
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Sheet {
    Run,
    Actions,
    Combat,
}
#[derive(Clone, Copy, PartialEq)]
struct Pose {
    sheet: Sheet,
    frame: usize,
    facing: i8,
}
pub struct Animation {
    stride: f32,
    time: f32,
    last_state: PlayerState,
    old_pose: Pose,
    blend: f32,
    last_pose: Pose,
}
impl Animation {
    pub fn new() -> Self {
        Self {
            stride: 0.0,
            time: 0.0,
            last_state: PlayerState::Falling,
            old_pose: Pose {
                sheet: Sheet::Actions,
                frame: 3,
                facing: -1,
            },
            last_pose: Pose {
                sheet: Sheet::Actions,
                frame: 3,
                facing: -1,
            },
            blend: 1.0,
        }
    }
    pub fn update(&mut self, p: &Player, delta: f32) {
        let old = self.last_pose;
        self.time += delta;
        self.stride += p.vx.abs() * delta / 46.0;
        if p.state != self.last_state {
            self.old_pose = old;
            self.blend = 0.0;
            self.last_state = p.state;
        }
        self.blend = (self.blend + delta / 0.09).min(1.0);
        self.last_pose = self.pose(p);
    }
    fn pose(&self, p: &Player) -> Pose {
        let (sheet, frame) = if p.state == PlayerState::Dead {
            (Sheet::Actions, 7)
        } else if p.landing_ticks > 3 && p.grounded && p.vx.abs() < 15.0 {
            (Sheet::Actions, 6)
        } else {
            match p.state {
                PlayerState::Running | PlayerState::Stepping => {
                    (Sheet::Run, (self.stride * 8.0) as usize % 8)
                }
                PlayerState::Jumping => (Sheet::Actions, 2),
                PlayerState::Falling => (Sheet::Actions, 3),
                PlayerState::Hanging => (Sheet::Actions, 4),
                PlayerState::Climbing => (Sheet::Actions, 5),
                PlayerState::Crouching => (Sheet::Actions, 1),
                PlayerState::Combat => (Sheet::Combat, 0),
                PlayerState::Attacking => (
                    Sheet::Combat,
                    if p.state_ticks < 5 {
                        1
                    } else if p.state_ticks < 12 {
                        2
                    } else {
                        4
                    },
                ),
                PlayerState::Parrying => (Sheet::Combat, 5),
                PlayerState::Hurt => (Sheet::Combat, 6),
                PlayerState::Idle | PlayerState::Stopping | PlayerState::Turning => {
                    (Sheet::Actions, 0)
                }
                PlayerState::Dead => (Sheet::Actions, 7),
            }
        };
        Pose {
            sheet,
            frame,
            facing: p.facing,
        }
    }
}

pub fn draw_scene(art: &Art, game: &Game, previous: &Player, alpha: f32, animation: &Animation) {
    let time = animation.time;
    let room = game.player.room;
    draw_texture_ex(
        &art.backdrop,
        0.0,
        0.0,
        Color::new(0.62, 0.66, 0.73, 1.0),
        DrawTextureParams {
            dest_size: Some(vec2(320.0, 189.0)),
            ..Default::default()
        },
    );
    draw_rectangle(0.0, 0.0, 320.0, 189.0, Color::new(0.02, 0.025, 0.04, 0.18));
    // Stone geometry is textured at a constant texel density, independently of physics.
    for index in 0..30 {
        let tile = game.tile(room, index);
        let runtime = game.runtime(room, index);
        let x = (index % COLS) as f32 * 32.0;
        let y = floor_y(index / COLS);
        if tile == Tile::Empty {
            continue;
        }
        if tile == Tile::Wall {
            masonry(
                art,
                Rect::new(x, y - 63.0, 32.0, 63.0),
                index,
                Color::new(0.86, 0.79, 0.69, 1.0),
            );
            draw_line(
                x,
                y - 62.6,
                x + 32.0,
                y - 62.6,
                0.3,
                Color::new(0.75, 0.6, 0.4, 0.28),
            );
            continue;
        }
        let shake = if runtime.loose_ticks > 0 {
            (time * 45.0).sin() * 0.4
        } else {
            0.0
        };
        masonry(
            art,
            Rect::new(x + shake, y, 32.0, 7.5),
            index,
            Color::new(0.95, 0.81, 0.62, 1.0),
        );
        draw_rectangle(x, y + 6.5, 32.0, 2.0, Color::new(0.02, 0.015, 0.01, 0.5));
        draw_line(
            x,
            y + 0.2,
            x + 32.0,
            y + 0.2,
            0.4,
            Color::new(0.88, 0.69, 0.4, 0.55),
        );
        match tile {
            Tile::Pillar => prop(art, 0, Rect::new(x + 6.0, y - 61.0, 35.0, 62.0), WHITE, 1.0),
            Tile::Torch => {
                glow(
                    x + 14.0,
                    y - 43.0,
                    26.0,
                    0.20 + (time * 7.0 + index as f32).sin() * 0.015,
                );
                prop(
                    art,
                    1,
                    Rect::new(x + 1.0, y - 53.0, 25.0, 29.0 + (time * 8.0).sin() * 0.25),
                    WHITE,
                    1.0,
                );
            }
            Tile::Gate => {
                let open = runtime.gate_open;
                if open < 0.99 {
                    prop(
                        art,
                        2,
                        Rect::new(x + 5.0, y - 61.0, 38.0, 62.0 * (1.0 - open)),
                        WHITE,
                        1.0 - open,
                    );
                }
            }
            Tile::Exit => prop(
                art,
                3,
                Rect::new(x - 7.0, y - 61.0, 72.0, 63.0),
                if runtime.gate_open > 0.9 {
                    WHITE
                } else {
                    Color::new(0.45, 0.43, 0.4, 1.0)
                },
                1.0,
            ),
            Tile::Sword if !game.player.sword => {
                prop(art, 4, Rect::new(x + 6.0, y - 18.0, 22.0, 20.0), WHITE, 1.0);
                crate::draw_crisp_text("SWORD", x + 3.0, y - 22.0, 7.0, GOLD);
                draw_triangle(
                    vec2(x + 14.0, y - 21.0),
                    vec2(x + 20.0, y - 21.0),
                    vec2(x + 17.0, y - 18.0),
                    GOLD,
                );
            }
            Tile::Potion if !runtime.consumed => {
                prop(art, 5, Rect::new(x + 6.0, y - 17.0, 20.0, 18.0), WHITE, 1.0)
            }
            Tile::Skeleton => {
                // Crop the atlas's large transparent margins so the skull and ribs
                // remain recognizable instead of flattening into a blade-like line.
                draw_texture_ex(
                    &art.props,
                    x - 1.0,
                    y - 12.0,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            art.props.width() * 0.503,
                            art.props.height() * 0.704,
                            art.props.width() * 0.244,
                            art.props.height() * 0.154,
                        )),
                        dest_size: Some(vec2(36.0, 12.0)),
                        ..Default::default()
                    },
                );
            }
            Tile::Spikes => prop(art, 7, Rect::new(x - 1.0, y - 14.0, 34.0, 16.0), WHITE, 1.0),
            Tile::Opener | Tile::Closer => {
                let tint = if runtime.pressed {
                    Color::new(1.0, 0.72, 0.32, 0.7)
                } else {
                    Color::new(0.5, 0.37, 0.23, 0.75)
                };
                draw_rectangle(x + 4.0, y - 0.8, 24.0, 1.3, tint);
            }
            Tile::Loose => draw_line(
                x + 12.0,
                y + 0.5,
                x + 17.0,
                y + 4.0,
                0.6,
                Color::new(0.06, 0.04, 0.025, 0.9),
            ),
            _ => {}
        }
    }
    for guard in &game.guards {
        if guard.room == room {
            let pose = Pose {
                sheet: if guard.health == 0 {
                    Sheet::Actions
                } else {
                    Sheet::Combat
                },
                frame: if guard.health == 0 {
                    7
                } else if guard.hurt > 0 {
                    6
                } else if guard.windup > 0 {
                    1
                } else {
                    0
                },
                facing: guard.facing,
            };
            shadow(guard.x, guard.y, 10.0, 0.27);
            actor(
                art,
                pose,
                vec2(guard.x, guard.y),
                Color::new(0.87, 0.52, 0.36, 1.0),
                1.0,
                0.0,
            );
            if guard.health > 0 {
                for i in 0..guard.health {
                    draw_rectangle(
                        guard.x - 5.0 + f32::from(i) * 4.0,
                        guard.y - 44.0,
                        3.0,
                        0.7,
                        Color::new(0.8, 0.2, 0.1, 0.8),
                    );
                }
            }
        }
    }
    let p = &game.player;
    let position = if previous.room == p.room && (p.x - previous.x).abs() < 80.0 {
        vec2(
            previous.x + (p.x - previous.x) * alpha,
            previous.y + (p.y - previous.y) * alpha,
        )
    } else {
        vec2(p.x, p.y)
    };
    shadow(
        position.x,
        position.y,
        8.0,
        if p.grounded { 0.3 } else { 0.12 },
    );
    let tint = if p.invulnerable > 0 {
        Color::new(1.0, 0.73, 0.62, 0.72 + (time * 32.0).sin() * 0.2)
    } else {
        Color::new(0.96, 0.91, 0.82, 1.0)
    };
    let pose = animation.pose(p);
    if animation.blend < 1.0 {
        actor(
            art,
            animation.old_pose,
            position,
            tint,
            1.0 - animation.blend,
            time,
        );
    }
    actor(art, pose, position, tint, animation.blend, time);
    if game.combat_feedback_ticks > 0 {
        let (label, color) = match game.last_combat_event {
            Event::Sword => ("SWORD READY", Color::new(1.0, 0.89, 0.6, 1.0)),
            Event::Parry => ("PARRY!", Color::new(0.6, 0.88, 1.0, 1.0)),
            Event::Hit => ("HIT", Color::new(1.0, 0.65, 0.35, 1.0)),
            _ => ("BLOCK", Color::new(1.0, 0.89, 0.6, 1.0)),
        };
        crate::draw_crisp_text(
            label,
            (position.x - 12.0).clamp(2.0, 278.0),
            position.y - 47.0,
            7.0,
            color,
        );
        let point = vec2(position.x + f32::from(p.facing) * 22.0, position.y - 25.0);
        let spread = f32::from(25 - game.combat_feedback_ticks) * 0.5;
        for i in 0..6 {
            let angle = i as f32 * 1.047;
            let direction = vec2(angle.cos(), angle.sin());
            let start = point + direction * spread;
            let end = start + direction * 3.0;
            draw_line(start.x, start.y, end.x, end.y, 0.6, color);
        }
    }
    // Dust motes and ember highlights are presentation effects, never collision objects.
    for i in 0..22 {
        let seed = i as f32 * 47.73 + f32::from(room) * 17.0;
        let x = (seed + time * (0.35 + (i % 3) as f32 * 0.2)).rem_euclid(320.0);
        let y = (seed * 2.37 - time * 0.4).rem_euclid(176.0) + 13.0;
        draw_circle(x, y, 0.12, Color::new(0.9, 0.73, 0.47, 0.17));
    }
    // Soft vignette; the center and landing surfaces stay readable.
    for i in 0..12 {
        let a = (12 - i) as f32 / 12.0 * 0.015;
        draw_rectangle(i as f32, 13.0, 1.0, 176.0, Color::new(0.0, 0.0, 0.0, a));
        draw_rectangle(
            319.0 - i as f32,
            13.0,
            1.0,
            176.0,
            Color::new(0.0, 0.0, 0.0, a),
        );
    }
    draw_rectangle(0.0, 189.0, 320.0, 11.0, Color::from_rgba(12, 11, 15, 255));
    let nearby_guard = game.guards.iter().any(|g| {
        g.health > 0 && g.room == p.room && (g.y - p.y).abs() < 16.0 && (g.x - p.x).abs() < 110.0
    });
    let nearby_sword = !p.sword
        && (0..30).any(|index| {
            game.tile(room, index) == Tile::Sword
                && (floor_y(index / COLS) - p.y).abs() < 8.0
                && ((index % COLS) as f32 * 32.0 + 16.0 - p.x).abs() < 44.0
        });
    let nearby_bones = !p.sword
        && (0..30).any(|index| {
            game.tile(room, index) == Tile::Skeleton
                && (floor_y(index / COLS) - p.y).abs() < 8.0
                && ((index % COLS) as f32 * 32.0 + 16.0 - p.x).abs() < 44.0
        });
    let hint = if nearby_sword {
        "Hold A / Z / SHIFT near the sword to pick it up"
    } else if nearby_guard && p.sword {
        "Z / SHIFT: Strike   X / UP: Block - time it to parry"
    } else if nearby_guard {
        "You need a sword. Retreat and find it, or try Sword practice."
    } else if nearby_bones {
        "Old bones. Find the weapon marked SWORD."
    } else if !p.sword {
        "Find the sword. Explore the western dungeon."
    } else {
        "Return east. Open the stairway and escape."
    };
    let width = measure_text(hint, None, 7, 1.0).width;
    draw_text(
        hint,
        (320.0 - width) / 2.0,
        196.0,
        7.0,
        Color::from_rgba(172, 154, 128, 255),
    );
}
fn masonry(art: &Art, dest: Rect, index: usize, tint: Color) {
    let sw = art.stone.width() / 4.0;
    let sh = art.stone.height() / 2.0;
    let source = Rect::new(
        (index % 4) as f32 * sw,
        (index / 4 % 2) as f32 * sh,
        sw,
        if dest.h < 12.0 { sh * 0.14 } else { sh },
    );
    draw_texture_ex(
        &art.stone,
        dest.x,
        dest.y,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(dest.w, dest.h)),
            source: Some(source),
            ..Default::default()
        },
    );
}
fn prop(art: &Art, index: usize, dest: Rect, tint: Color, visible: f32) {
    let w = art.props.width() / 4.0;
    let h = art.props.height() / 2.0;
    draw_texture_ex(
        &art.props,
        dest.x,
        dest.y,
        tint,
        DrawTextureParams {
            source: Some(Rect::new(
                (index % 4) as f32 * w,
                (index / 4) as f32 * h,
                w,
                h * visible,
            )),
            dest_size: Some(vec2(dest.w, dest.h)),
            ..Default::default()
        },
    );
}
fn actor(art: &Art, pose: Pose, position: Vec2, tint: Color, opacity: f32, time: f32) {
    let texture = match pose.sheet {
        Sheet::Run => &art.run,
        Sheet::Actions => &art.actions,
        Sheet::Combat => &art.combat,
    };
    let w = texture.width() / 4.0;
    let h = texture.height() / 2.0;
    let baseline = match pose.sheet {
        Sheet::Run => [427.0, 432.0, 431.0, 428.0, 424.0, 425.0, 425.0, 425.0][pose.frame] / 443.5,
        Sheet::Actions => {
            [438.0, 428.0, 387.0, 437.0, 436.0, 433.0, 411.0, 407.0][pose.frame] / 443.5
        }
        Sheet::Combat => {
            [424.0, 423.0, 417.0, 416.0, 398.0, 397.0, 398.0, 398.0][pose.frame] / 443.5
        }
    };
    let height = if pose.sheet == Sheet::Combat {
        57.0
    } else if pose.sheet == Sheet::Actions && pose.frame == 0 {
        43.0 + (time * 2.5).sin() * 0.13
    } else {
        43.0
    };
    let color = Color::new(tint.r, tint.g, tint.b, tint.a * opacity);
    let extra = if pose.sheet == Sheet::Combat && pose.frame == 2 {
        26.0
    } else {
        0.0
    };
    draw_texture_ex(
        texture,
        position.x - height / 2.0,
        position.y - height * baseline,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(height * (w + extra) / w, height)),
            source: Some(Rect::new(
                (pose.frame % 4) as f32 * w,
                (pose.frame / 4) as f32 * h,
                w + extra,
                h,
            )),
            flip_x: pose.facing < 0,
            ..Default::default()
        },
    );
}
fn shadow(x: f32, y: f32, width: f32, alpha: f32) {
    for i in 0..5 {
        draw_ellipse(
            x,
            y + 0.9,
            width - i as f32,
            1.5 - i as f32 * 0.2,
            0.0,
            Color::new(0.0, 0.0, 0.0, alpha * 0.13),
        );
    }
}
fn glow(x: f32, y: f32, radius: f32, strength: f32) {
    for i in (1..18).rev() {
        let r = radius * i as f32 / 18.0;
        draw_circle(x, y, r, Color::new(1.0, 0.46, 0.12, strength / 18.0));
    }
}
