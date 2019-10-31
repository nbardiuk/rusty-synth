use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::f32::consts::PI;
use std::time::Duration;

const TAU: f32 = PI * 2.0;

struct Synth {
    time: f32,
    time_step: f32,
    volume: f32,
}

impl Synth {
    fn noise(&self, time: f32) -> f32 {
        self.volume * (440.0 * TAU * time).sin()
    }
}

impl AudioCallback for Synth {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            self.time += self.time_step;
            *x = self.noise(self.time);
        }
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(512),
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // initialize the audio callback
        println!("{:?}", spec);
        Synth {
            time: 0.0,
            time_step: 1.0 / spec.freq as f32,
            volume: 0.15,
        }
    })?;

    // Start playback
    device.resume();

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
