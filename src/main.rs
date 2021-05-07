use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ndarray::prelude::*;

use rayon::prelude::*;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

mod render;
mod env;

use env::EuclidianRaytracing;

// Screen size
const SIZE: u32 = 32;
const SCREEN: [u32;2] = [SIZE, SIZE];
const SCALE: u32 = 16;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("TEST", SCALE*SCREEN[0], SCALE*SCREEN[1])
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .unwrap();

    canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
    canvas.clear();
    canvas.present();

    // Camera
    let camera = render::Camera::new(
        EuclidianRaytracing::new(array![0.0,0.0,0.0], array![0.0,0.0,-1.0]),
    );

    // Synchronization
    let a_ended = Arc::new(Mutex::new(false));
    let a_ended_thread = a_ended.clone();

    // Pixels 
    let a_pixels = Arc::new(Mutex::new(vec![Color::RGB(0xff, 0xff, 0xff);(SCREEN[0]*SCREEN[1]) as usize]));
   
    let a_pixels_thread = a_pixels.clone();

    // Render thread
    let render_thread = thread::spawn(move || {
        (0..SCREEN[0]*SCREEN[1]).into_par_iter().for_each(|ii| {
            // Check if quit
            {
                let ended_thread = a_ended_thread.lock().unwrap();
                if *ended_thread {
                    return;
                }
            }

            let i = ii / SCREEN[0];
            let j = ii % SCREEN[0];
            let pixel_color = camera.render_pixel(j, i, SCREEN);
            {
                let mut pixels = a_pixels_thread.lock().unwrap();
                pixels[(i*SCREEN[0] + j) as usize] = pixel_color;
            }
        });
    });

    // Main loop
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => (),
            }
        }
        
        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x10));
        canvas.clear();
        
        // Rendering
        {
            let pixels = a_pixels.lock().unwrap();
            for ii in 0..pixels.len() {
                let ii = ii as u32;
                let i = (ii / SCREEN[0]) * SCALE;
                let j = (ii % SCREEN[0]) * SCALE;
                
                canvas.set_draw_color(pixels[ii as usize]);
                canvas.fill_rect(Rect::new(j as i32, i as i32, SCALE, SCALE)).unwrap();
            }

        }

        canvas.present();
        thread::sleep(Duration::new(0, (1e9 as u32) / 60));
    }
    

    // Close render thread
    {
        let mut ended = a_ended.lock().unwrap();
        *ended = true;
    }
    render_thread.join().unwrap();
}
