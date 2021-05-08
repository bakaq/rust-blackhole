use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration};

use clap::{App, load_yaml};

use image;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;


mod render;
mod env;

use env::{EuclidianRaytracing};


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
    
    // Parameters
    let spinning: f64 = matches.value_of("spinning").unwrap_or("0").parse().unwrap();
    
    let skydome = match matches.value_of("skydome") {
        Some(path) => {
            match image::open(path) {
                Ok(image) => {
                    Some(Box::new(image.into_rgb8()))
                },
                Err(_) => None,
            }
        },
        None => None,
    };

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
    let r = 4.0;
    let mut phi = 0.0;
    let camera = render::Camera::new(EuclidianRaytracing::new_orbiting_spherical(
            (r, std::f64::consts::FRAC_PI_2, phi), aspect, skydome.clone()));

    // Synchronization
    let a_running = Arc::new(Mutex::new(false));

    // Pixels 
    let a_pixels = Arc::new(Mutex::new(vec![Color::RGB(0xff, 0xff, 0xff);(screen[0]*screen[1]) as usize]));

    // Renderer
    let mut renderer = render::Renderer::new(screen, camera, a_pixels.clone(), a_running.clone());
    renderer.start_render();

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
            let running = {
                let running = a_running.lock().unwrap();
                *running
            };
            if !running {
                phi += spinning;
                let camera_new = render::Camera::new(EuclidianRaytracing::new_orbiting_spherical(
                    (r, (1.0 + 0.5*(phi*4.0).sin())*std::f64::consts::FRAC_PI_2, phi), aspect, skydome.clone()));
                renderer.camera = camera_new;
                renderer.start_render()
            }
        }

        canvas.present();
        thread::sleep(Duration::new(0, (1e9 as u32) / 60));
    }
}

/*
struct Renderer<T> where
    T: Environment
{
    screen: [u32; 2],
    camera: render::Camera<T>,
    pixels: Arc<Mutex<Vec<Color>>>,
    running: Arc<Mutex<bool>>,
    render_thread: Option<thread::JoinHandle<()>>,
}

impl<T> Renderer<T> where
    T: Environment
{
    fn new(screen: [u32;2], camera: render::Camera<T>, pixels: Arc<Mutex<Vec<Color>>>, running: Arc<Mutex<bool>>) -> Renderer<T> {
        let render_thread = None;
        Renderer {screen, camera, pixels, running, render_thread}
    }

    fn start_render(&mut self) {
        println!("Start render!");
        self.stop_render();

        let a_running = self.running.clone();
        let a_pixels = self.pixels.clone();
        let screen = self.screen.clone(); 
        let camera = self.camera.clone();

        self.render_thread = Some(thread::spawn(move || {
            let t0 = Instant::now();
            (0..screen[0]*screen[1]).into_par_iter().for_each(|ii| {
                // Check if quit
                {
                    let running = a_running.lock().unwrap();
                    if !*running {
                        return;
                    }
                }

                let i = ii / screen[0];
                let j = ii % screen[0];
                let pixel_color = camera.render_pixel(j, i, screen);
                {
                    let mut pixels = a_pixels.lock().unwrap();
                    pixels[(i*screen[0] + j) as usize] = pixel_color;
                }
            });

            {
                let mut running = a_running.lock().unwrap();
                *running = false;
            }

            println!("Finished rendering in {}s", t0.elapsed().as_nanos() as f64 / 1e9 );
        }));
        
        {
            let mut running = self.running.lock().unwrap();
            *running = true;
        }
    }

    fn stop_render(&mut self) {
        if let Some(join_handle) = self.render_thread.take() {
            {
                let mut running = self.running.lock().unwrap();
                *running = false;
            }
            join_handle.join().unwrap();
        } 
    }
}

impl<T> Drop for Renderer<T> where
    T: Environment
{
    fn drop(&mut self) {
        self.stop_render();
    }
}
*/
