use chip8_rs::chip8::{Chip8, CHIP8_SCREEN_HEIGHT, CHIP8_SCREEN_WIDTH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{pixels, EventPump};
use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;

const BLACK: pixels::Color = pixels::Color::RGB(0, 0, 0);
const WHITE: pixels::Color = pixels::Color::RGB(255, 255, 255);

pub struct Chip8Sdl {
    canvas: Canvas<Window>,
    chip8: Chip8,
    screen_scale_factor: u32,
    events: EventPump,
    keys: [bool; 16],
    sleep_duration: u64,
}

impl Chip8Sdl {
    pub fn new(sleep_duration: u64, scale: u32, path: impl AsRef<Path>) -> Self {
        let mut chip8 = Chip8::new();
        chip8.load_rom(path);

        let sdl = sdl2::init().unwrap();
        let events = sdl.event_pump().unwrap();
        let video_subsys = sdl.video().unwrap();
        let window = video_subsys
            .window(
                "Chip8",
                CHIP8_SCREEN_WIDTH as u32 * scale,
                CHIP8_SCREEN_HEIGHT as u32 * scale,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self {
            canvas,
            events,
            chip8,
            screen_scale_factor: scale,
            keys: [false; 16],
            sleep_duration,
        }
    }

    pub fn draw(&mut self) {
        for (r, row) in self.chip8.screen.iter().enumerate() {
            let y = (r as u32) * self.screen_scale_factor;
            for (c, col) in row.iter().enumerate() {
                let x = (c as u32) * self.screen_scale_factor;
                let color = if *col == 0 { BLACK } else { WHITE };
                self.canvas.set_draw_color(color);
                let _ = self.canvas.fill_rect(Rect::new(
                    x as i32,
                    y as i32,
                    self.screen_scale_factor,
                    self.screen_scale_factor,
                ));
            }
        }
        self.canvas.present();
    }

    pub fn poll(&mut self) -> Result<(), ()> {
        for event in self.events.poll_iter() {
            //println!("{:?}", event);
            match event {
                Event::Quit { .. } => return Err(()),
                _ => (),
            };
        }

        let keys: Vec<Keycode> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        //println!("{:?}", keys);
        for key in keys.iter() {
            match key {
                Keycode::Num1 => self.keys[1] = true,
                Keycode::Num2 => self.keys[2] = true,
                Keycode::Num3 => self.keys[3] = true,
                Keycode::Num4 => self.keys[0xc] = true,
                Keycode::Q => self.keys[4] = true,
                Keycode::W => self.keys[5] = true,
                Keycode::E => self.keys[6] = true,
                Keycode::R => self.keys[0xd] = true,
                Keycode::A => self.keys[7] = true,
                Keycode::S => self.keys[8] = true,
                Keycode::D => self.keys[9] = true,
                Keycode::F => self.keys[0xe] = true,
                Keycode::Z => self.keys[0xa] = true,
                Keycode::X => self.keys[0] = true,
                Keycode::C => self.keys[0xb] = true,
                Keycode::V => self.keys[0xf] = true,
                _ => (),
            };
        }
        if keys.len() == 0 {
            self.keys = [false; 16];
        }
        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            self.poll().unwrap();
            self.chip8.cycle(self.keys);
            self.draw();
            thread::sleep(Duration::from_millis(self.sleep_duration));
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut chip8sdl = Chip8Sdl::new(
        args[1].parse::<u64>().unwrap(),
        args[2].parse::<u32>().unwrap(),
        args[3].clone(),
    );
    chip8sdl.run();
}
