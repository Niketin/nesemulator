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
        .window("nesemulator", 1536, 960)
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
    let mut texture_game = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 256, 240)
        .unwrap();
    let mut texture_pattern_table_0 = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 128, 128)
        .unwrap();
    let mut texture_pattern_table_1 = texture_creator
        .create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 128, 128)
        .unwrap();
    let game_rect = Some(sdl2::rect::Rect::new(0, 0, 1024, 960));
    let pattern_table_0_rect = Some(sdl2::rect::Rect::new(1024, 0, 512, 512));
    let pattern_table_1_rect = Some(sdl2::rect::Rect::new(1024, 512, 512, 512));
    let mut pattern_table_0_pixels = emulator::ppu::display::Display::new(128, 128);
    let mut pattern_table_1_pixels = emulator::ppu::display::Display::new(128, 128);

    'running: loop {
        emulator.step();
        let ppu = &mut emulator.cpu.bus.ppu.as_mut().unwrap();

        // Check if there is a need to render a frame
        if ppu.y == 240 && waiting_to_render {
            let display = &ppu.display;

            let _width = display.width;
            let _height = display.height;
            let pixels = display.get_pixels();

            // Update game screen
            texture_game.update(None, pixels, 256 * 3)
                .expect("Main loop: failed to update the texture");
            canvas.copy(&texture_game, None, game_rect)
                .expect("Main loop: failed to copy the texture in to canvas: {}");

            // Update pattern table

            ppu.load_pattern_table_tiles_to_display(0x0000, &mut pattern_table_0_pixels);
            ppu.load_pattern_table_tiles_to_display(0x1000, &mut pattern_table_1_pixels);


            texture_pattern_table_0.update(None, pattern_table_0_pixels.get_pixels(), 128 * 3)
                .expect("Main loop: failed to update the texture");
            texture_pattern_table_1.update(None, pattern_table_1_pixels.get_pixels(), 128 * 3)
                .expect("Main loop: failed to update the texture");
            canvas.copy(&texture_pattern_table_0, None, pattern_table_0_rect)
                .expect("Main loop: failed to copy the texture in to canvas: {}");
            canvas.copy(&texture_pattern_table_1, None, pattern_table_1_rect)
                .expect("Main loop: failed to copy the texture in to canvas: {}");

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