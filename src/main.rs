extern crate sdl2;

use emulator::{Emulator, Button};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut emulator = Emulator::new(&args[1]);
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("nesemulator", 1024, 960)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> =
        window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut waiting_to_render = true;

    let texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext> =
        canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    'running: loop {
        emulator.step();
        let ppu = &mut emulator.cpu.bus.ppu.as_mut().unwrap();

        // Check if there is a need to render a frame
        if ppu.y == 240 && waiting_to_render {
            let display = &ppu.display;

            let _width = display.width;
            let _height = display.height;
            let pixels = display.get_pixels();
            if let Err(e) = texture.update(None, pixels, 256 * 3) {
                panic!("Main loop: failed to update the texture: {}", e);
            }
            if let Err(e) = canvas.copy(&texture, None, None) {
                panic!("Main loop: failed to copy the texture in to canvas: {}", e);
            }
            canvas.present();
            waiting_to_render = false;

        }
        if ppu.y == 241 {
            waiting_to_render = true;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {..} | Event::KeyUp {..} => handle_emulator_input(event, &mut emulator),
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn handle_emulator_input(event: Event, emulator: &mut Emulator) {
    let value = match event {
        Event::KeyDown { repeat: true, ..} => return,
        Event::KeyUp { repeat: true, ..} => return,
        Event::KeyDown {..} => true,
        _ => false,
    };

    let button = match event {
        Event::KeyDown { keycode: Some(Keycode::Z), .. } => Button::A,
        Event::KeyDown { keycode: Some(Keycode::X), .. } => Button::B,
        Event::KeyDown { keycode: Some(Keycode::N), .. } => Button::Start,
        Event::KeyDown { keycode: Some(Keycode::M), .. } => Button::Select,
        Event::KeyDown { keycode: Some(Keycode::Up), .. } => Button::Up,
        Event::KeyDown { keycode: Some(Keycode::Down), .. } => Button::Down,
        Event::KeyDown { keycode: Some(Keycode::Left), .. } => Button::Left,
        Event::KeyDown { keycode: Some(Keycode::Right), .. } => Button::Right,

        Event::KeyUp { keycode: Some(Keycode::Z), .. } => Button::A,
        Event::KeyUp { keycode: Some(Keycode::X), .. } => Button::B,
        Event::KeyUp { keycode: Some(Keycode::N), .. } => Button::Start,
        Event::KeyUp { keycode: Some(Keycode::M), .. } => Button::Select,
        Event::KeyUp { keycode: Some(Keycode::Up), .. } => Button::Up,
        Event::KeyUp { keycode: Some(Keycode::Down), .. } => Button::Down,
        Event::KeyUp { keycode: Some(Keycode::Left), .. } => Button::Left,
        Event::KeyUp { keycode: Some(Keycode::Right), .. } => Button::Right,
        _ => return
    };

    emulator.set_controller_state(button, value);
}