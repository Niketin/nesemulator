extern crate sdl2; 

use emulator::Emulator;
use std::env;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut emulator = Emulator::new(&args[1]);
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    
    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().build().unwrap();
 
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    println!("Entering the main loop.");

    let mut waiting_to_render = true;

    let texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext> = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB888, 256, 240).unwrap();

    

    'running: loop {
        emulator.cpu.step();
        print!("Program counter: {}.    ", emulator.cpu.program_counter);
        let ppu = &mut emulator.cpu.bus.ppu.as_mut().unwrap();
        println!("PPU scanline: {:4}, cycle: {:4}", ppu.scanline, ppu.cycle);
        ppu.step();
        ppu.step();
        ppu.step();
        
        // Check if there is a need to render a frame
        if ppu.cycle == 240 && waiting_to_render {

            let display = &ppu.display;

            let width = display.width;
            let height = display.height;
            let pixels = display.get_pixels();
            if let Err(e) = texture.update(None, pixels, 256) {
                panic!("Main loop: failed to update the texture: {}", e);
            }
            if let Err(e) = canvas.copy(&texture, None, None) {
                panic!("Main loop: failed to copy the texture in to canvas: {}", e);
            }
            waiting_to_render = false;
        }
        if ppu.cycle == 241 {
            waiting_to_render = true;
        }

        
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

}
