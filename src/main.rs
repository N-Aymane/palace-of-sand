//! Pixel renderer and browser/native platform adapter.
use macroquad::audio::{Sound, load_sound_from_bytes, play_sound_once};
use macroquad::prelude::*;
use palace_of_sand::level::Level;
use palace_of_sand::{DT, Event, Game, Input, Phase};
mod visual;
use visual::{Animation, Art};

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
        platform: miniquad::conf::Platform {
            webgl_version: miniquad::conf::WebGLVersion::WebGL2,
            ..Default::default()
        },
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
    let art = match Art::load().await {
        Ok(art) => art,
        Err(error) => loop {
            clear_background(INK);
            draw_text(&error, 12.0, 35.0, 16.0, RED);
            next_frame().await;
        },
    };
    let mut animation = Animation::new();
    let mut previous = game.player.clone();
    let desktop_target = render_target(1280, 800);
    let touch_target = render_target(1280, 1056);
    desktop_target.texture.set_filter(FilterMode::Linear);
    touch_target.texture.set_filter(FilterMode::Linear);
    let mut audio: Option<Audio> = None;
    let mut audio_attempted = false;
    let mut muted = false;
    let mut screen = Screen::Title;
    let mut accumulator = 0.0;
    let mut previous_input = Input::default();
    let mut pending_up = false;
    let mut pending_action = false;
    macroquad::logging::info!("PALACE_GRAPHICS_READY");
    loop {
        let touch_layout = screen_width() / screen_height() < 1.45;
        let virtual_h = if touch_layout { 264.0 } else { 200.0 };
        let target = if touch_layout {
            &touch_target
        } else {
            &desktop_target
        };
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
        pending_up |= input.up && !previous_input.up;
        pending_action |= input.action && !previous_input.action;
        previous_input = input;
        if screen == Screen::Playing && focused {
            accumulator += get_frame_time().min(0.1);
            while accumulator >= DT {
                previous = game.player.clone();
                game.step(Input {
                    up: input.up || pending_up,
                    action: input.action || pending_action,
                    ..input
                });
                pending_up = false;
                pending_action = false;
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
            pending_up = false;
            pending_action = false;
            previous = game.player.clone();
        }
        animation.update(
            &game.player,
            if screen == Screen::Playing {
                get_frame_time().min(0.1)
            } else {
                0.0
            },
        );
        set_camera(&Camera2D {
            render_target: Some(target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, 320.0, virtual_h))
        });
        clear_background(INK);
        visual::draw_scene(
            &art,
            &game,
            &previous,
            (accumulator / DT).clamp(0.0, 1.0),
            &animation,
        );
        draw_hud(&game, muted, screen == Screen::Paused);
        if screen != Screen::Playing || game.phase != Phase::Playing {
            draw_overlay(&game, screen);
        }
        if touch_layout {
            draw_touch_controls(input);
        }
        set_default_camera();
        clear_background(Color::from_rgba(9, 9, 14, 255));
        draw_texture_ex(
            &target.texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(320.0 * scale, virtual_h * scale)),
                flip_y: true,
                ..Default::default()
            },
        );
        next_frame().await;
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
