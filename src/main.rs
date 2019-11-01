use rand::prelude::*;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

const TAU: f32 = PI * 2.0;

struct Synth<'a> {
    time: f32,
    time_step: f32,
    volume: f32,
    mu_hertz: &'a AtomicU32,
}

impl<'a> Synth<'a> {
    fn play(&self, time: f32) -> f32 {
        let hertz = self.mu_hertz.load(Ordering::Relaxed) as f32 / 1_000000.;
        self.volume * Synth::noise(hertz, time)
    }

    fn sine(hertz: f32, time: f32) -> f32 {
        (hertz * TAU * time).sin()
    }

    fn square(hertz: f32, time: f32) -> f32 {
        if (hertz * TAU * time).sin() > 0. {
            1.
        } else {
            -1.
        }
    }

    fn triangle(hertz: f32, time: f32) -> f32 {
        (hertz * TAU * time).sin().asin() * 2. / PI
    }

    fn saw(hertz: f32, time: f32) -> f32 {
        let x = hertz * TAU * time;
        let mut result = 0.;
        for n in 1..40 {
            result += (x * n as f32).sin() / n as f32
        }
        result * 2. / PI
    }

    fn noise(hertz: f32, time: f32) -> f32 {
        if hertz == 0. {
            0.
        } else {
            thread_rng().gen_range(-1.0, 1.0)
        }
    }
}

impl<'a> AudioCallback for Synth<'a> {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            self.time += self.time_step;
            *x = self.play(self.time);
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(512),
    };

    let note = AtomicU32::new(0);
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            println!("{:?}", spec);
            Synth {
                time: 0.0,
                time_step: 1.0 / spec.freq as f32,
                volume: 0.15,
                mu_hertz: &note,
            }
        })
        .unwrap();

    let keyboard = [
        //A
        (Keycode::Q, 440_000000),
        (Keycode::Num2, 466_163800),
        (Keycode::W, 493_883300),
        //C
        (Keycode::E, 523_251100),
        (Keycode::Num4, 554_365300),
        (Keycode::R, 587_329500),
        (Keycode::Num5, 622_254000),
        (Keycode::T, 659_255100),
        //F
        (Keycode::Y, 698_456500),
        (Keycode::Num7, 739_988800),
        (Keycode::U, 783_990900),
        (Keycode::Num8, 830_609400),
        //A
        (Keycode::I, 880_000000),
        (Keycode::Num9, 932_327500),
        (Keycode::O, 987_766600),
        //C
        (Keycode::P, 1046_502000),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();

    let width = 100;
    let height = 100;
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("render preview", width as u32, height as u32)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    let mut view = window.into_canvas().build().unwrap();
    view.set_logical_size(width as u32, height as u32).unwrap();

    // Start playback
    device.resume();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } if keyboard.contains_key(&key) => {
                    if let Some(&v) = keyboard.get(&key) {
                        note.store(v, Ordering::Relaxed);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } if keyboard.contains_key(&key) => {
                    note.store(0, Ordering::Relaxed);
                }
                _ => {}
            }
        }
        view.present();
        thread::sleep(Duration::from_secs_f32(1. / 120.));
    }
}
