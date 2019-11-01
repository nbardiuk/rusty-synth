use rand::prelude::*;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

const TAU: f32 = PI * 2.0;

struct Synth<'a> {
    time: &'a mut f32,
    time_step: f32,
    hertz: &'a f32,
    envelope: &'a Adsr,
}

struct Adsr {
    start_amplitude: f32,
    attack_time: f32,
    decay_time: f32,
    sustain_amplitude: f32,
    release_time: f32,

    trigger_start: Option<f32>,
    trigger_end: Option<f32>,
}

impl Adsr {
    fn amplitude(&self, time: f32) -> f32 {
        let amplitude = if let Some(start_time) = self.trigger_start {
            match self.trigger_end {
                None => {
                    // button on
                    let local_time = time - start_time;
                    if local_time <= self.attack_time {
                        // phase attack
                        (local_time / self.attack_time) * self.start_amplitude
                    } else if local_time <= self.attack_time + self.decay_time {
                        // phase decay
                        (local_time - self.attack_time) / self.decay_time
                            * (self.sustain_amplitude - self.start_amplitude)
                            + self.start_amplitude
                    } else {
                        // phase sustain
                        self.sustain_amplitude
                    }
                }
                Some(end_time) => {
                    // button off
                    // phase release
                    (time - end_time) / self.release_time * (0. - self.sustain_amplitude)
                        + self.sustain_amplitude
                }
            }
        } else {
            0.
        };

        if amplitude <= 0.0001 {
            0.
        } else {
            amplitude
        }
    }
}

impl<'a> Synth<'a> {
    fn play(&self, time: f32) -> f32 {
        self.envelope.amplitude(time) * (Synth::lfo(Synth::saw, *self.hertz, time, 5.0, 0.001))
    }

    fn lfo(osc: fn(f32) -> f32, hertz: f32, time: f32, lfo_hertz: f32, lfo_amplitude: f32) -> f32 {
        osc(hertz * (TAU * time + lfo_amplitude * (lfo_hertz * TAU * time).sin()))
    }

    fn sine(w: f32) -> f32 {
        w.sin()
    }

    fn square(w: f32) -> f32 {
        if w.sin() > 0. {
            1.
        } else {
            -1.
        }
    }

    fn triangle(w: f32) -> f32 {
        w.sin().asin() * 2. / PI
    }

    fn saw(w: f32) -> f32 {
        let mut result = 0.;
        for n in 1..40 {
            result += (w * n as f32).sin() / n as f32
        }
        result * 2. / PI
    }

    fn noise(w: f32) -> f32 {
        if w == 0. {
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
            *self.time += self.time_step;
            *x = self.play(*self.time);
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

    let mut note = 0.;
    let mut envelope = Adsr {
        start_amplitude: 0.2,
        attack_time: 0.1,
        decay_time: 0.01,
        sustain_amplitude: 0.18,
        release_time: 0.2,

        trigger_start: None,
        trigger_end: None,
    };
    let mut time = 0.;
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            println!("{:?}", spec);
            Synth {
                time: &mut time,
                time_step: 1.0 / spec.freq as f32,
                hertz: &note,
                envelope: &envelope,
            }
        })
        .unwrap();

    let keyboard = [
        //A
        (Keycode::Q, 440.),
        (Keycode::Num2, 466.1638),
        (Keycode::W, 493.8833),
        //C
        (Keycode::E, 523.2511),
        (Keycode::Num4, 554.3653),
        (Keycode::R, 587.3295),
        (Keycode::Num5, 622.254),
        (Keycode::T, 659.2551),
        //F
        (Keycode::Y, 698.4565),
        (Keycode::Num7, 739.9888),
        (Keycode::U, 783.9909),
        (Keycode::Num8, 830.6094),
        //A
        (Keycode::I, 880.),
        (Keycode::Num9, 932.3275),
        (Keycode::O, 987.7666),
        //C
        (Keycode::P, 1046.502),
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
                        if !(note == v && envelope.trigger_end.is_none()) {
                            note = v;
                            envelope.trigger_start = Some(time);
                            envelope.trigger_end = None;
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } if keyboard.contains_key(&key) => {
                    envelope.trigger_end = Some(time);
                }
                _ => {}
            }
        }
        view.present();
        thread::sleep(Duration::from_secs_f32(1. / 120.));
    }
}
