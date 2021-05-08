use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};

use rayon::prelude::*;

use sdl2::pixels::Color;

use crate::env::Environment;

#[derive(Clone)]
pub struct Camera<T> where
    T: Environment
{
    env: T,
}

impl<T> Camera<T> where
    T: Environment
{
    pub fn new(env: T) -> Camera<T> {
        Camera{env}
    }

    pub fn render_pixel(&self, x: u32, y: u32, size: [u32; 2]) -> Color {
        let x = x as f64 + 0.5;
        let y = y as f64 + 0.5;

        let sw = size[0] as f64;
        let sh = size[1] as f64;

        let x = (x - sw/2.0)/(sw/2.0);
        let y = (sh/2.0 - y)/(sh/2.0);
       
        self.env.raytrace((x,y))
    }
}

pub struct Renderer<T> where
    T: Environment
{
    screen: [u32; 2],
    pub camera: Camera<T>,
    pixels: Arc<Mutex<Vec<Color>>>,
    running: Arc<Mutex<bool>>,
    render_thread: Option<thread::JoinHandle<()>>,
}

impl<T> Renderer<T> where
    T: Environment
{
    pub fn new(screen: [u32;2], camera: Camera<T>, pixels: Arc<Mutex<Vec<Color>>>, running: Arc<Mutex<bool>>) -> Renderer<T> {
        let render_thread = None;
        Renderer {screen, camera, pixels, running, render_thread}
    }

    pub fn start_render(&mut self) {
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

    pub fn stop_render(&mut self) {
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
