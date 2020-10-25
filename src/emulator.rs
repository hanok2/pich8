use crate::cpu::CPU;
use crate::display::WindowDisplay;
use crate::contracts::{DisplayOutput, SoundOutput};
use crate::sound::{NoSound, BeepSound};
use bitvec::prelude::*;
use spin_sleep::sleep;
use std::time::{Duration, Instant};
use sdl2::{
    EventPump,
    event::Event,
    keyboard::Keycode,
};

pub struct Emulator<T: SoundOutput> {
    cpu: CPU,
    display: WindowDisplay,
    sound: T,
    input: BitArray<Msb0, [u16; 1]>,
    event_pump: EventPump,
}

impl Emulator<BeepSound> {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init().unwrap();
        Ok(Self{
            cpu: CPU::new(),
            display: WindowDisplay::new(&sdl_context, true)?,
            sound: BeepSound::new(&sdl_context)?,
            input: bitarr![Msb0, u16; 0; 16],
            event_pump: sdl_context.event_pump()?,
        })
    }
}

impl Emulator<NoSound> {
    pub fn new_without_sound() -> Result<Self, String> {
        let sdl_context = sdl2::init().unwrap();
        Ok(Self{
            cpu: CPU::new(),
            display: WindowDisplay::new(&sdl_context, true)?,
            sound: NoSound{},
            input: bitarr![Msb0, u16; 0; 16],
            event_pump: sdl_context.event_pump()?,
        })
    }
}

impl<T: SoundOutput> Emulator<T> {
    const FRAMES_PER_SEC: u64 = 60;
    const CYCLES_PER_FRAME: u16 = 10;
    const NANOS_PER_FRAME: u64 = 1_000_000_000 / Emulator::<T>::FRAMES_PER_SEC;

    pub fn run(&mut self, rom: &[u8]) {
        self.cpu.load_rom(rom);
        self.run_loop();
    }

    pub fn run_state(&mut self, state: &[u8]) -> Result<(), String> {
        self.cpu = CPU::from_state(state).map_err(|e| format!("error loading state: {}", e))?;
        self.run_loop();
        Ok(())
    }

    fn run_loop(&mut self) {
        loop {
            let frame_start = Instant::now();

            if self.handle_events() {
                break;
            }

            for _ in 0..Emulator::<T>::CYCLES_PER_FRAME {
                self.cpu.tick(&self.input);
                if self.cpu.sound_active() {
                    self.sound.beep();
                }
            }
            self.cpu.update_timers();
            self.display.draw(self.cpu.vmem()).expect("failed to render frame");

            self.sleep(&frame_start);
        }
    }

    fn sleep(&mut self, frame_start: &Instant) {
        let sleep_time = Emulator::<T>::NANOS_PER_FRAME as f64 - frame_start.elapsed().as_nanos() as f64;
        if sleep_time > 0.0 {
            sleep(Duration::from_nanos(sleep_time as u64));
        }
    }

    fn handle_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit{..} => return true,
                Event::KeyDown{ keycode: Some(Keycode::Escape), .. } => return true,
                Event::KeyDown{ keycode: Some(Keycode::F11), .. } => { self.display.toggle_fullscreen().unwrap(); },

                // Chip8 Keys
                Event::KeyDown{ keycode: Some(Keycode::Num1), .. } => self.input.set(0, true),
                Event::KeyDown{ keycode: Some(Keycode::Num2), .. } => self.input.set(1, true),
                Event::KeyDown{ keycode: Some(Keycode::Num3), .. } => self.input.set(2, true),
                Event::KeyDown{ keycode: Some(Keycode::Num4), .. } => self.input.set(3, true),
                Event::KeyDown{ keycode: Some(Keycode::Q), .. } => self.input.set(4, true),
                Event::KeyDown{ keycode: Some(Keycode::W), .. } => self.input.set(5, true),
                Event::KeyDown{ keycode: Some(Keycode::E), .. } => self.input.set(6, true),
                Event::KeyDown{ keycode: Some(Keycode::R), .. } => self.input.set(7, true),
                Event::KeyDown{ keycode: Some(Keycode::A), .. } => self.input.set(8, true),
                Event::KeyDown{ keycode: Some(Keycode::S), .. } => self.input.set(9, true),
                Event::KeyDown{ keycode: Some(Keycode::D), .. } => self.input.set(10, true),
                Event::KeyDown{ keycode: Some(Keycode::F), .. } => self.input.set(11, true),
                Event::KeyDown{ keycode: Some(Keycode::Y), .. } => self.input.set(12, true),
                Event::KeyDown{ keycode: Some(Keycode::X), .. } => self.input.set(13, true),
                Event::KeyDown{ keycode: Some(Keycode::C), .. } => self.input.set(14, true),
                Event::KeyDown{ keycode: Some(Keycode::V), .. } => self.input.set(15, true),
                Event::KeyUp{ keycode: Some(Keycode::Num1), .. } => self.input.set(0, false),
                Event::KeyUp{ keycode: Some(Keycode::Num2), .. } => self.input.set(1, false),
                Event::KeyUp{ keycode: Some(Keycode::Num3), .. } => self.input.set(2, false),
                Event::KeyUp{ keycode: Some(Keycode::Num4), .. } => self.input.set(3, false),
                Event::KeyUp{ keycode: Some(Keycode::Q), .. } => self.input.set(4, false),
                Event::KeyUp{ keycode: Some(Keycode::W), .. } => self.input.set(5, false),
                Event::KeyUp{ keycode: Some(Keycode::E), .. } => self.input.set(6, false),
                Event::KeyUp{ keycode: Some(Keycode::R), .. } => self.input.set(7, false),
                Event::KeyUp{ keycode: Some(Keycode::A), .. } => self.input.set(8, false),
                Event::KeyUp{ keycode: Some(Keycode::S), .. } => self.input.set(9, false),
                Event::KeyUp{ keycode: Some(Keycode::D), .. } => self.input.set(10, false),
                Event::KeyUp{ keycode: Some(Keycode::F), .. } => self.input.set(11, false),
                Event::KeyUp{ keycode: Some(Keycode::Y), .. } => self.input.set(12, false),
                Event::KeyUp{ keycode: Some(Keycode::X), .. } => self.input.set(13, false),
                Event::KeyUp{ keycode: Some(Keycode::C), .. } => self.input.set(14, false),
                Event::KeyUp{ keycode: Some(Keycode::V), .. } => self.input.set(15, false),
                _ => {}
            }
        }
        false
    }
}