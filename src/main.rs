//! Pixel renderer and browser/native platform adapter.
use macroquad::audio::{Sound, load_sound_from_bytes, play_sound_once};
use macroquad::prelude::*;
use palace_of_sand::level::{COLS, Level, Tile, floor_y};
use palace_of_sand::{DT, Event, Game, Input, Phase, PlayerState};

const INK: Color = Color::from_rgba(14, 14, 23, 255);
const STONE: Color = Color::from_rgba(62, 59, 77, 255);
const EDGE: Color = Color::from_rgba(110, 101, 113, 255);
const GOLD: Color = Color::from_rgba(218, 177, 102, 255);
const LIGHT: Color = Color::from_rgba(235, 222, 191, 255);

fn conf() -> Conf {
    Conf {
        window_title: "Palace of Sand".into(),
        window_width: 960,
        window_height: 600,
        high_dpi: true,
        sample_count: 1,
        ..Default::default()
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Title,
    Playing,
    Paused,
    Help,
}
struct Audio {
    step: Sound,
    jump: Sound,
    metal: Sound,
    hurt: Sound,
    win: Sound,
}
impl Audio {
    async fn new() -> Option<Self> {
        Some(Self {
            step: load_sound_from_bytes(&tone(&[90.0], 0.045, 0.12))
                .await
                .ok()?,
            jump: load_sound_from_bytes(&tone(&[180.0, 240.0], 0.08, 0.10))
                .await
                .ok()?,
            metal: load_sound_from_bytes(&tone(&[540.0, 810.0, 1080.0], 0.04, 0.09))
                .await
                .ok()?,
            hurt: load_sound_from_bytes(&tone(&[130.0, 95.0, 65.0], 0.10, 0.14))
                .await
                .ok()?,
            win: load_sound_from_bytes(&tone(&[261.63, 329.63, 392.0, 523.25], 0.16, 0.12))
                .await
                .ok()?,
        })
    }
    fn play(&self, e: Event) {
        play_sound_once(match e {
            Event::Step | Event::Land | Event::Crumble => &self.step,
            Event::Jump | Event::Grab => &self.jump,
            Event::Sword | Event::Gate | Event::Clash => &self.metal,
            Event::Hurt | Event::Death => &self.hurt,
            Event::Complete | Event::Drink => &self.win,
        });
    }
}
fn tone(notes: &[f32], duration: f32, volume: f32) -> Vec<u8> {
    let rate = 22050u32;
    let samples = (duration * rate as f32) as usize;
    let size = (samples * notes.len() * 2) as u32;
    let mut wav = Vec::with_capacity(size as usize + 44);
    wav.extend(b"RIFF");
    wav.extend((size + 36).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(rate.to_le_bytes());
    wav.extend((rate * 2).to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(size.to_le_bytes());
    for &frequency in notes {
        for i in 0..samples {
            let t = i as f32 / rate as f32;
            let envelope = (1.0 - i as f32 / samples as f32) * (t * 150.0).min(1.0);
            let wave = (t * frequency * std::f32::consts::TAU).sin()
                + 0.22 * (t * frequency * 2.0 * std::f32::consts::TAU).sin();
            wav.extend(((wave * envelope * volume * 32767.0) as i16).to_le_bytes());
        }
    }
    wav
}

#[macroquad::main(conf)]
async fn main() {
    let level = match Level::opening() {
        Ok(l) => l,
        Err(error) => loop {
            clear_background(INK);
            draw_text(&error, 20.0, 60.0, 18.0, RED);
            next_frame().await;
        },
    };
    let mut game = Game::new(level);
    let target = render_target(320, 200);
    target.texture.set_filter(FilterMode::Nearest);
    let mut audio: Option<Audio> = None;
    let mut audio_attempted = false;
    let mut muted = false;
    let mut screen = Screen::Title;
    let mut accumulator = 0.0;
    loop {
        let touch_layout = screen_width() / screen_height() < 1.45;
        let virtual_h = if touch_layout { 264.0 } else { 200.0 };
        let scale = (screen_width() / 320.0).min(screen_height() / virtual_h);
        let offset = vec2(
            (screen_width() - 320.0 * scale) / 2.0,
            (screen_height() - virtual_h * scale) / 2.0,
        );
        let pointer = |x: f32, y: f32| vec2((x - offset.x) / scale, (y - offset.y) / scale);
        let pointers: Vec<Vec2> = touches()
            .iter()
            .filter(|t| !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled))
            .map(|t| pointer(t.position.x, t.position.y))
            .collect();
        let mouse = pointer(mouse_position().0, mouse_position().1);
        let click = is_mouse_button_pressed(MouseButton::Left)
            || touches().iter().any(|t| t.phase == TouchPhase::Started);
        let hit = |r: Rect| {
            (click && r.contains(mouse))
                || touches().iter().any(|t| {
                    t.phase == TouchPhase::Started
                        && r.contains(pointer(t.position.x, t.position.y))
                })
        };
        let held = |r: Rect| {
            pointers.iter().any(|p| r.contains(*p))
                || (is_mouse_button_down(MouseButton::Left) && r.contains(mouse))
        };
        // The shell sends F12 on visibility loss, after releasing held keys.
        let focused = !is_key_pressed(KeyCode::F12);
        if !focused && screen == Screen::Playing {
            screen = Screen::Paused;
            accumulator = 0.0;
        }
        if is_key_pressed(KeyCode::M) || hit(Rect::new(269.0, 0.0, 22.0, 13.0)) {
            muted = !muted;
        }
        if is_key_pressed(KeyCode::R) || hit(Rect::new(240.0, 0.0, 26.0, 13.0)) {
            game.restart();
            screen = Screen::Playing;
            accumulator = 0.0;
        }
        if is_key_pressed(KeyCode::Escape) || hit(Rect::new(294.0, 0.0, 26.0, 13.0)) {
            screen = if screen == Screen::Playing {
                Screen::Paused
            } else if screen == Screen::Paused || screen == Screen::Help {
                Screen::Playing
            } else {
                screen
            };
            accumulator = 0.0;
        }
        if screen == Screen::Title
            || screen == Screen::Paused
            || screen == Screen::Help
            || game.phase != Phase::Playing
        {
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space)
                || hit(Rect::new(90.0, 125.0, 140.0, 23.0))
            {
                if game.phase != Phase::Playing {
                    game.restart();
                }
                screen = Screen::Playing;
                accumulator = 0.0;
                if !audio_attempted {
                    audio = Audio::new().await;
                    audio_attempted = true;
                }
            }
            if hit(Rect::new(110.0, 153.0, 100.0, 19.0)) {
                screen = if screen == Screen::Help {
                    Screen::Title
                } else {
                    Screen::Help
                };
            }
        }
        let input = if screen == Screen::Playing && focused {
            Input {
                left: is_key_down(KeyCode::Left) || held(Rect::new(8.0, 226.0, 34.0, 32.0)),
                right: is_key_down(KeyCode::Right) || held(Rect::new(80.0, 226.0, 34.0, 32.0)),
                up: is_key_down(KeyCode::Up) || held(Rect::new(44.0, 205.0, 34.0, 26.0)),
                down: is_key_down(KeyCode::Down) || held(Rect::new(44.0, 234.0, 34.0, 28.0)),
                action: is_key_down(KeyCode::LeftShift)
                    || is_key_down(KeyCode::RightShift)
                    || held(Rect::new(253.0, 211.0, 58.0, 47.0)),
            }
        } else {
            Input::default()
        };
        if screen == Screen::Playing && focused {
            accumulator += get_frame_time().min(0.1);
            while accumulator >= DT {
                // Keep press edges until a simulation tick consumes them.
                game.step(input);
                if !muted {
                    if let Some(sounds) = &audio {
                        for &event in &game.events {
                            sounds.play(event);
                        }
                    }
                }
                accumulator -= DT;
            }
        } else {
            accumulator = 0.0;
        }
        set_camera(&Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, 320.0, 200.0))
        });
        clear_background(INK);
        draw_room(&game);
        draw_hud(&game, muted, screen == Screen::Paused);
        if screen != Screen::Playing || game.phase != Phase::Playing {
            draw_overlay(&game, screen);
        }
        set_default_camera();
        clear_background(Color::from_rgba(9, 9, 14, 255));
        draw_texture_ex(
            &target.texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(320.0 * scale, 200.0 * scale)),
                flip_y: true,
                ..Default::default()
            },
        );
        if touch_layout {
            set_camera(&Camera2D::from_display_rect(Rect::new(
                -offset.x / scale,
                -offset.y / scale,
                screen_width() / scale,
                screen_height() / scale,
            )));
            draw_touch_controls(input);
            set_default_camera();
        }
        next_frame().await;
    }
}

fn draw_room(game: &Game) {
    let room = game.player.room;
    // Masonry is generated from integer room/brick coordinates, never gameplay RNG.
    draw_rectangle(0.0, 13.0, 320.0, 176.0, Color::from_rgba(24, 23, 35, 255));
    for by in 0..12 {
        for bx in -1..11 {
            let hash = ((bx + 7) * 173 + by * 37 + i32::from(room) * 59) as u32;
            let x = bx as f32 * 32.0 + if by % 2 == 0 { 0.0 } else { 16.0 };
            let y = by as f32 * 16.0 + 13.0;
            let v = (hash % 9) as u8;
            let color = Color::from_rgba(29 + v, 28 + v, 40 + v, 255);
            draw_rectangle(x, y, 31.0, 15.0, color);
            draw_line(
                x + 2.0,
                y + 1.0,
                x + 29.0,
                y + 1.0,
                1.0,
                Color::from_rgba(46, 43, 57, 255),
            );
            if hash % 7 == 0 {
                draw_line(
                    x + 12.0,
                    y + 3.0,
                    x + 15.0,
                    y + 9.0,
                    1.0,
                    Color::from_rgba(19, 18, 29, 255),
                );
            }
        }
    }
    for c in [2, 7] {
        arch(c as f32 * 32.0 + 5.0, 79.0);
    }
    for index in 0..30 {
        let tile = game.tile(room, index);
        let x = (index % COLS) as f32 * 32.0;
        let y = floor_y(index / COLS);
        let rt = game.runtime(room, index);
        if tile == Tile::Wall {
            wall(x, y - TILE_H_SAFE(), 32.0, 63.0);
            continue;
        }
        if tile == Tile::Empty {
            if rt.removed && rt.loose_ticks == 0 && game.level.tile(room, index) == Tile::Loose {
                draw_line(x + 2.0, y + 3.0, x + 28.0, y + 4.0, 1.0, STONE);
            }
            continue;
        }
        let shake = if rt.loose_ticks > 0 {
            if game.tick % 6 < 3 { -1.0 } else { 1.0 }
        } else {
            0.0
        };
        draw_rectangle(x + shake, y, 32.0, 8.0, STONE);
        draw_rectangle(x + shake, y, 32.0, 2.0, EDGE);
        draw_line(
            x + 2.0,
            y + 6.0,
            x + 30.0,
            y + 6.0,
            1.0,
            Color::from_rgba(39, 36, 50, 255),
        );
        draw_line(x, y, x, y + 7.0, 1.0, INK);
        match tile {
            Tile::Pillar => {
                draw_rectangle(x + 22.0, y - 58.0, 6.0, 57.0, STONE);
                draw_rectangle(x + 23.0, y - 57.0, 2.0, 56.0, EDGE);
                draw_rectangle(x + 18.0, y - 59.0, 15.0, 5.0, EDGE);
                draw_rectangle(x + 19.0, y - 5.0, 12.0, 5.0, EDGE);
            }
            Tile::Torch => torch(x + 14.0, y - 40.0, game.tick, index),
            Tile::Loose => {
                draw_line(x + 9.0, y + 1.0, x + 14.0, y + 5.0, 1.0, INK);
                draw_line(x + 14.0, y + 5.0, x + 22.0, y + 2.0, 1.0, INK);
            }
            Tile::Spikes => {
                for i in 0..5 {
                    let sx = x + 4.0 + i as f32 * 6.0;
                    draw_triangle(
                        vec2(sx, y),
                        vec2(sx + 2.0, y - 10.0),
                        vec2(sx + 4.0, y),
                        Color::from_rgba(174, 165, 161, 255),
                    );
                }
            }
            Tile::Gate => {
                let top = y - 59.0;
                let bottom = y - rt.gate_open * 58.0;
                draw_rectangle(x + 3.0, top, 28.0, 3.0, EDGE);
                for bar in 0..5 {
                    let bx = x + 6.0 + bar as f32 * 5.0;
                    draw_rectangle(
                        bx,
                        top,
                        2.0,
                        (bottom - top).max(0.0),
                        Color::from_rgba(119, 111, 117, 255),
                    );
                }
                draw_rectangle(x + 4.0, bottom, 27.0, 3.0, EDGE);
            }
            Tile::Opener | Tile::Closer => {
                draw_rectangle(
                    x + 5.0,
                    y - 2.0,
                    23.0,
                    3.0,
                    if rt.pressed {
                        GOLD
                    } else {
                        Color::from_rgba(150, 123, 89, 255)
                    },
                );
                draw_rectangle(x + 10.0, y - 1.0, 12.0, 1.0, INK);
            }
            Tile::Sword if !game.player.sword => {
                draw_line(x + 9.0, y - 3.0, x + 25.0, y - 9.0, 2.0, LIGHT);
                draw_line(x + 7.0, y - 6.0, x + 11.0, y, 2.0, GOLD);
                if game.tick % 90 < 20 {
                    draw_line(x + 24.0, y - 14.0, x + 24.0, y - 6.0, 1.0, WHITE);
                    draw_line(x + 20.0, y - 10.0, x + 28.0, y - 10.0, 1.0, WHITE);
                }
            }
            Tile::Potion if !rt.consumed => {
                draw_rectangle(
                    x + 13.0,
                    y - 8.0,
                    7.0,
                    7.0,
                    Color::from_rgba(149, 63, 69, 255),
                );
                draw_rectangle(x + 15.0, y - 12.0, 3.0, 5.0, GOLD);
                draw_rectangle(x + 14.0, y - 7.0, 2.0, 3.0, LIGHT);
            }
            Tile::Exit => {
                draw_rectangle(x - 2.0, y - 54.0, 62.0, 54.0, STONE);
                draw_rectangle(x + 3.0, y - 50.0, 51.0, 50.0, INK);
                let amount = game.runtime(room, index).gate_open;
                for step in 0..5 {
                    draw_rectangle(
                        x + 5.0,
                        y - step as f32 * 5.0,
                        48.0 - step as f32 * 5.0,
                        3.0,
                        EDGE,
                    );
                }
                draw_rectangle(
                    x + 4.0,
                    y - 49.0,
                    49.0,
                    49.0 * (1.0 - amount),
                    Color::from_rgba(93, 74, 61, 255),
                );
                for line in 0..5 {
                    let ly = y - 48.0 + line as f32 * 10.0;
                    if ly < y - 49.0 * amount {
                        draw_line(x + 5.0, ly, x + 51.0, ly, 1.0, GOLD);
                    }
                }
            }
            Tile::Skeleton => {
                for i in 0..4 {
                    draw_line(
                        x + 8.0 + i as f32 * 4.0,
                        y - 3.0,
                        x + 10.0 + i as f32 * 4.0,
                        y - 6.0,
                        1.0,
                        EDGE,
                    );
                }
                draw_rectangle(x + 5.0, y - 6.0, 5.0, 5.0, LIGHT);
            }
            Tile::Debris => {
                for i in 0..4 {
                    draw_rectangle(
                        x + 4.0 + i as f32 * 6.0,
                        y - 2.0 - (i % 2) as f32,
                        4.0,
                        3.0,
                        EDGE,
                    );
                }
            }
            _ => {}
        }
    }
    for guard in &game.guards {
        if guard.room == room {
            person(
                guard.x,
                guard.y,
                guard.facing,
                if guard.health == 0 {
                    PlayerState::Dead
                } else if guard.windup > 0 {
                    PlayerState::Attacking
                } else {
                    PlayerState::Combat
                },
                game.tick,
                guard.windup,
                true,
                true,
                guard.hurt > 0,
            );
            if guard.health > 0 {
                for i in 0..guard.health {
                    draw_rectangle(
                        guard.x - 7.0 + f32::from(i) * 6.0,
                        guard.y - 47.0,
                        4.0,
                        2.0,
                        Color::from_rgba(188, 88, 72, 255),
                    );
                }
            }
        }
    }
    let p = &game.player;
    if p.invulnerable == 0 || game.tick % 6 < 4 {
        person(
            p.x,
            p.y,
            p.facing,
            p.state,
            game.tick,
            p.state_ticks,
            p.sword,
            false,
            false,
        );
    }
    // Darkness at the edges keeps the warm torch highlights legible.
    for y in 0..4 {
        draw_rectangle(
            0.0,
            185.0 + y as f32,
            320.0,
            1.0,
            Color::new(0.03, 0.03, 0.05, 0.15 * y as f32),
        );
    }
    draw_rectangle(0.0, 189.0, 320.0, 11.0, INK);
    let hint = if !p.sword {
        "FIND THE SWORD  /  EXPLORE TO THE LEFT"
    } else {
        "RETURN EAST  /  OPEN THE EXIT"
    };
    center(hint, 196.0, 7.0, Color::from_rgba(144, 130, 118, 255));
}
#[allow(non_snake_case)]
fn TILE_H_SAFE() -> f32 {
    63.0
}
fn wall(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, Color::from_rgba(51, 47, 62, 255));
    for row in 0..4 {
        let yy = y + row as f32 * 16.0;
        draw_line(x, yy, x + w, yy, 1.0, Color::from_rgba(28, 26, 37, 255));
        draw_line(
            x + 1.0,
            yy + 1.0,
            x + w - 1.0,
            yy + 1.0,
            Color::from_rgba(83, 74, 88, 255).a,
            EDGE,
        );
        let xx = x + if row % 2 == 0 { 12.0 } else { 25.0 };
        draw_line(xx, yy, xx, yy + 15.0, 1.0, INK);
    }
}
fn arch(x: f32, y: f32) {
    draw_rectangle(x, y - 30.0, 22.0, 30.0, Color::from_rgba(19, 18, 29, 255));
    for i in 0..6 {
        draw_rectangle(
            x + i as f32 * 2.0,
            y - 30.0 - (i.min(5 - i)) as f32 * 3.0,
            2.0,
            4.0,
            STONE,
        );
    }
    draw_line(x - 2.0, y - 28.0, x - 2.0, y + 1.0, 2.0, STONE);
    draw_line(x + 24.0, y - 28.0, x + 24.0, y + 1.0, 2.0, STONE);
}
fn torch(x: f32, y: f32, tick: u64, index: usize) {
    for r in (1..4).rev() {
        draw_circle(
            x,
            y - 4.0,
            r as f32 * 7.0,
            Color::new(0.8, 0.36, 0.08, 0.018),
        );
    }
    draw_rectangle(x - 2.0, y, 4.0, 10.0, Color::from_rgba(104, 72, 51, 255));
    draw_rectangle(x - 5.0, y + 7.0, 10.0, 2.0, STONE);
    let flicker = ((tick / 7 + index as u64) % 3) as f32;
    draw_rectangle(
        x - 3.0,
        y - 8.0 - flicker,
        6.0,
        9.0 + flicker,
        Color::from_rgba(198, 89, 39, 255),
    );
    draw_rectangle(x - 2.0, y - 9.0 + flicker, 4.0, 8.0 - flicker, GOLD);
    draw_rectangle(x - 1.0, y - 4.0, 2.0, 5.0, LIGHT);
}
#[allow(clippy::too_many_arguments)]
fn person(
    x: f32,
    y: f32,
    facing: i8,
    state: PlayerState,
    tick: u64,
    state_ticks: u16,
    sword: bool,
    guard: bool,
    hurt: bool,
) {
    let d = f32::from(facing);
    let walking = matches!(
        state,
        PlayerState::Running | PlayerState::Stepping | PlayerState::Combat
    );
    let phase = (tick as f32 * 0.25).sin();
    let stride = if walking { phase * 6.0 } else { 2.0 };
    let crouch = matches!(state, PlayerState::Crouching | PlayerState::Hanging);
    let dead = state == PlayerState::Dead;
    let hip = if crouch { y - 10.0 } else { y - 18.0 };
    let cloth = if hurt {
        Color::from_rgba(232, 120, 91, 255)
    } else if guard {
        Color::from_rgba(175, 91, 66, 255)
    } else {
        Color::from_rgba(218, 208, 180, 255)
    };
    let shade = if guard {
        Color::from_rgba(111, 53, 51, 255)
    } else {
        Color::from_rgba(147, 141, 141, 255)
    };
    let skin = Color::from_rgba(205, 151, 112, 255);
    if dead {
        draw_rectangle(x - 15.0, y - 6.0, 25.0, 5.0, cloth);
        draw_rectangle(x + 10.0 * d, y - 7.0, 7.0, 6.0, skin);
        return;
    }
    draw_line(x - 2.0, hip, x - stride, y - 2.0, 5.0, shade);
    draw_line(x + 2.0, hip, x + stride, y - 2.0, 5.0, cloth);
    draw_rectangle(
        x - stride - 3.0,
        y - 3.0,
        7.0,
        3.0,
        Color::from_rgba(91, 70, 66, 255),
    );
    draw_rectangle(
        x + stride - 2.0,
        y - 3.0,
        7.0,
        3.0,
        Color::from_rgba(91, 70, 66, 255),
    );
    let torso_y = hip - 13.0;
    draw_rectangle(x - 5.0, torso_y, 10.0, 15.0, cloth);
    draw_rectangle(
        x - 5.0,
        hip - 1.0,
        10.0,
        3.0,
        if guard {
            GOLD
        } else {
            Color::from_rgba(146, 67, 63, 255)
        },
    );
    draw_rectangle(x - 3.0, torso_y - 8.0, 7.0, 8.0, skin);
    draw_rectangle(
        x - 4.0,
        torso_y - 10.0,
        8.0,
        4.0,
        Color::from_rgba(56, 37, 38, 255),
    );
    draw_rectangle(x + 3.0 * d, torso_y - 6.0, 2.0, 2.0, INK);
    let combat = matches!(
        state,
        PlayerState::Combat | PlayerState::Attacking | PlayerState::Parrying
    );
    let hand_x = x + d * if combat { 12.0 } else { 5.0 };
    let hand_y = if state == PlayerState::Hanging {
        y - 38.0
    } else if state == PlayerState::Parrying {
        torso_y - 9.0
    } else {
        torso_y + 8.0
    };
    draw_line(x + d * 3.0, torso_y + 4.0, hand_x, hand_y, 3.0, skin);
    if sword && (combat || guard) {
        let thrust = if state == PlayerState::Attacking {
            ((f32::from(state_ticks).min(14.0) / 14.0) * std::f32::consts::PI).sin() * 15.0
        } else {
            0.0
        };
        let end = vec2(
            hand_x + d * (17.0 + thrust),
            hand_y
                - if state == PlayerState::Parrying {
                    17.0
                } else {
                    4.0
                },
        );
        draw_line(hand_x, hand_y, end.x, end.y, 2.0, LIGHT);
        draw_line(
            hand_x - d * 2.0,
            hand_y - 4.0,
            hand_x + d * 2.0,
            hand_y + 4.0,
            2.0,
            GOLD,
        );
    }
}
fn draw_hud(game: &Game, muted: bool, paused: bool) {
    draw_rectangle(0.0, 0.0, 320.0, 13.0, INK);
    draw_line(
        0.0,
        12.0,
        320.0,
        12.0,
        1.0,
        Color::from_rgba(83, 66, 54, 255),
    );
    for i in 0..3 {
        let x = 8.0 + i as f32 * 9.0;
        draw_triangle(
            vec2(x, 4.0),
            vec2(x + 6.0, 4.0),
            vec2(x + 3.0, 10.0),
            if i < game.player.health as usize {
                Color::from_rgba(203, 103, 80, 255)
            } else {
                STONE
            },
        );
    }
    draw_text("I / THE DUNGEON", 44.0, 9.0, 8.0, GOLD);
    let remaining = 3600 - game.elapsed_ticks / 60;
    center(
        &format!("{:02}:{:02}", remaining / 60, remaining % 60),
        9.0,
        9.0,
        LIGHT,
    );
    draw_text("RST", 241.0, 9.0, 7.0, EDGE);
    draw_text(if muted { "OFF" } else { "SND" }, 270.0, 9.0, 7.0, EDGE);
    draw_text(if paused { ">" } else { "II" }, 300.0, 9.0, 9.0, GOLD);
}
fn center(text: &str, y: f32, size: f32, color: Color) {
    let width = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, (320.0 - width) / 2.0, y, size, color);
}
fn draw_overlay(game: &Game, screen: Screen) {
    draw_rectangle(
        0.0,
        13.0,
        320.0,
        176.0,
        Color::new(0.025, 0.023, 0.04, 0.88),
    );
    draw_line(93.0, 39.0, 146.0, 39.0, 1.0, GOLD);
    draw_line(174.0, 39.0, 227.0, 39.0, 1.0, GOLD);
    draw_poly(160.0, 39.0, 4, 4.0, 0.0, GOLD);
    let (eyebrow, title, subtitle, button) = if game.phase == Phase::Dead {
        (
            "THE DUNGEON CLAIMS ANOTHER",
            "A fallen prince",
            "Every escape begins with another attempt.",
            "TRY AGAIN",
        )
    } else if game.phase == Phase::Complete {
        (
            "THE FIRST DOOR OPENS",
            "Into the light",
            "The dungeon is behind you. The story has begun.",
            "PLAY AGAIN",
        )
    } else {
        match screen {
            Screen::Title => (
                "A CINEMATIC PLATFORMER",
                "Palace of Sand",
                "A sword. A way out. Sixty minutes.",
                "ENTER THE DUNGEON",
            ),
            Screen::Paused => (
                "TAKE A BREATH",
                "Time stands still",
                "Your escape can wait a moment.",
                "RESUME",
            ),
            Screen::Help => (
                "THE ART OF SURVIVAL",
                "Move with intention",
                "",
                "BEGIN YOUR ESCAPE",
            ),
            Screen::Playing => ("", "", "", ""),
        }
    };
    center(eyebrow, 57.0, 8.0, GOLD);
    center(title, 83.0, 25.0, LIGHT);
    if screen == Screen::Help {
        center("ARROWS  Move / Jump / Crouch", 99.0, 9.0, LIGHT);
        center(
            "SHIFT  Step carefully / Grab / Pick up / Strike",
            110.0,
            8.0,
            EDGE,
        );
        center(
            "UP  Climb a ledge / Parry / Enter the exit",
            120.0,
            8.0,
            EDGE,
        );
    } else {
        center(subtitle, 102.0, 9.0, EDGE);
    }
    draw_rectangle(90.0, 129.0, 140.0, 20.0, GOLD);
    center(button, 142.0, 10.0, INK);
    center(
        if screen == Screen::Help {
            "BACK TO TITLE"
        } else {
            "HOW TO PLAY"
        },
        165.0,
        8.0,
        GOLD,
    );
    center(
        "RUST / WEBASSEMBLY  -  AN UNOFFICIAL TRIBUTE",
        182.0,
        6.0,
        EDGE,
    );
}
fn draw_touch_controls(input: Input) {
    draw_rectangle(0.0, 200.0, 320.0, 64.0, INK);
    let buttons = [
        (Rect::new(8.0, 226.0, 34.0, 32.0), "<", input.left),
        (Rect::new(80.0, 226.0, 34.0, 32.0), ">", input.right),
        (Rect::new(44.0, 205.0, 34.0, 26.0), "^", input.up),
        (Rect::new(44.0, 234.0, 34.0, 28.0), "v", input.down),
        (Rect::new(253.0, 211.0, 58.0, 47.0), "ACTION", input.action),
    ];
    for (r, label, pressed) in buttons {
        draw_rectangle(
            r.x,
            r.y,
            r.w,
            r.h,
            if pressed {
                GOLD
            } else {
                Color::from_rgba(35, 32, 43, 255)
            },
        );
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, if pressed { LIGHT } else { STONE });
        let size = if label.len() > 1 { 9.0 } else { 16.0 };
        let width = measure_text(label, None, size as u16, 1.0).width;
        draw_text(
            label,
            r.x + (r.w - width) / 2.0,
            r.y + r.h / 2.0 + size * 0.35,
            size,
            if pressed { INK } else { GOLD },
        );
    }
    draw_text("HOLD ACTION", 137.0, 226.0, 7.0, EDGE);
    draw_text("TO STEP SAFELY", 132.0, 237.0, 7.0, EDGE);
}
