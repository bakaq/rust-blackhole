use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use rayon::prelude::*;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use clap::{App, load_yaml};

mod render;
mod env;

use env::{Environment, EuclidianRaytracing};

fn render_frame<T: 'static>(screen: [u32;2], camera: render::Camera<T>,
                a_ended_thread: Arc<Mutex<bool>>,
                a_pixels_thread: Arc<Mutex<Vec<Color>>>)
                -> thread::JoinHandle<()>
    where T: Environment + Send + Sync
{
    thread::spawn(move || {
        let t0 = Instant::now();
        (0..screen[0]*screen[1]).into_par_iter().for_each(|ii| {
            // Check if quit
            {
                let ended_thread = a_ended_thread.lock().unwrap();
                if *ended_thread {
                    return;
                }
            }

            let i = ii / screen[0];
            let j = ii % screen[0];
            let pixel_color = camera.render_pixel(j, i, screen);
            {
                let mut pixels = a_pixels_thread.lock().unwrap();
                pixels[(i*screen[0] + j) as usize] = pixel_color;
            }
        });

        {
            let mut ended_thread = a_ended_thread.lock().unwrap();
            *ended_thread = true;
        }

        println!("Finished rendering in {}s", t0.elapsed().as_nanos() as f64 / 1e9 );
    })
}

fn main() {
    // == Deal with CLI arguments ==
    let args_file = load_yaml!("args.yaml");
    let matches = App::from_yaml(args_file).get_matches();

    // Screen
    let scale: u32 = matches.value_of("scale").unwrap_or("1").parse().unwrap();
    
    let screen_raw = matches.value_of("screen").unwrap_or("400");
    let screen: [u32;2] = match screen_raw.parse::<u32>() {
        Ok(screen) => [screen, screen],
        Err(_) => {
            let screen = &screen_raw.split("x").map(|x| {
                x.parse::<u32>().unwrap()
            }).collect::<Vec<u32>>()[..2];
            [screen[0], screen[1]]
        }
    };

    let aspect = screen[0] as f64 / screen[1] as f64;

    let spinning: f64 = matches.value_of("spinning").unwrap_or("0").parse().unwrap();

    // SDL2 stuff
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("TEST", scale*screen[0], scale*screen[1])
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
    let mut phi = 0.0;
    let camera = render::Camera::new(EuclidianRaytracing::new_orbiting_spherical(
            (2.0, std::f64::consts::FRAC_PI_2, phi), aspect));

    // Synchronization
    let a_ended = Arc::new(Mutex::new(false));

    // Pixels 
    let a_pixels = Arc::new(Mutex::new(vec![Color::RGB(0xff, 0xff, 0xff);(screen[0]*screen[1]) as usize]));

    // Render thread
    let mut render_thread;

    render_thread = render_frame(screen, camera, a_ended.clone(), a_pixels.clone());
    
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
                let i = (ii / screen[0]) * scale;
                let j = (ii % screen[0]) * scale;
                
                canvas.set_draw_color(pixels[ii as usize]);
                canvas.fill_rect(Rect::new(j as i32, i as i32, scale, scale)).unwrap();
            }

        }
        
        if spinning != 0.0 {
            let mut ended = a_ended.lock().unwrap();
            if *ended {
                phi += spinning;
                let camera_new = render::Camera::new(EuclidianRaytracing::new_orbiting_spherical(
                    (2.0, (1.0 + 0.5*(phi*4.0).sin())*std::f64::consts::FRAC_PI_2, phi), aspect));
                render_thread = render_frame(screen, camera_new, a_ended.clone(), a_pixels.clone());
                *ended = false;
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
