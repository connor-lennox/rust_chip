extern crate sdl2;
use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard:: {Keycode, Scancode};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;
use std::env;
use std::path::Path;

use crate::cpu::Cpu;

mod cpu;
mod display;
mod keypad;

const CYCLES_PER_SECOND: u32 = 700;
const CYCLES_PER_TIMER: u32 = CYCLES_PER_SECOND / 60;
const CYCLE_DELAY: u32 = 1e9 as u32 / CYCLES_PER_SECOND;

fn main() {
    // Initialize emulator CPU
    let mut c: Cpu = cpu::Cpu::new();

    // Load ch8 cartridge file into CPU ram
    let args: Vec<String> = env::args().collect();
    let cart_path = if args.len() > 1 {Path::new(&args[1])} else {Path::new("/hdd/Documents/Programming/Rust/rust_chip/carts/test_opcode.ch8")};
    match c.read_program(cart_path) {
        Ok(_) => println!("Loaded cartidge into RAM"),
        Err(_) => panic!("Could not open cartridge file!"),
    }

    let mut cycle_count: u32 = 0;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Rust CHIP-8", 64 * 8, 32 * 8)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    match canvas.set_scale(8.0, 8.0) {
        Err(_) => panic!("error setting SDL scale on canvas"),
        Ok(_) => (),
    }

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        let keys: Vec<usize> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(|c| scancode_to_keypad(c))
            .collect();
        
        // println!("{:?}", keys);
        c.keypad.set_state(keys);

        // The rest of the game loop goes here...
        c.do_cycle();
        if c.draw_flag {
            render_to_screen(&mut canvas, &c);
        }

        cycle_count += 1;
        if cycle_count % CYCLES_PER_TIMER == 0 {
            c.delay = if c.delay > 0 {c.delay - 1} else {c.delay};
            c.sound = if c.sound > 0 {c.sound - 1} else {c.sound};
            cycle_count = 0;
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, CYCLE_DELAY));
    }
}

fn render_to_screen(canvas: &mut Canvas<Window>, cpu: &Cpu) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for y in 0..32 {
        for x in 0..64 {
            if cpu.display.get_pixel(x, y) {
                match canvas.draw_point(Point::new(x as i32, y as i32)) {
                    Err(_) => panic!("error rendering to screen"),
                    Ok(_) => (),
                }
            }
        }
    }
}

fn scancode_to_keypad(code: Scancode) -> Option<usize> {
    match code {
        Scancode::Num1 => Some(0),
        Scancode::Num2 => Some(1),
        Scancode::Num3 => Some(2),
        Scancode::Q => Some(3),
        Scancode::W => Some(4),
        Scancode::E => Some(5),
        Scancode::A => Some(6),
        Scancode::S => Some(7),
        Scancode::D => Some(8),
        Scancode::X => Some(9),
        Scancode::Z => Some(10),
        Scancode::C => Some(11),
        Scancode::Num4 => Some(12),
        Scancode::R => Some(13),
        Scancode::F => Some(14),
        Scancode::V => Some(15),
        _ => None
    }
}
