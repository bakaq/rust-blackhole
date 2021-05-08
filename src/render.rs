use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};

use rayon::prelude::*;

use sdl2::pixels::Color;

use crate::env::Environment;

pub trait Renderer<T> where
    T: Environment
{
    fn start_render(&mut self);
    
    fn get_pixels(&self) -> Vec<Color>;

    fn is_ready(&self) -> bool;
}

pub struct RayonRenderer<T> where
    T: Environment
{
    screen: [u32; 2],
    pub env: T,
    pixels: Arc<Mutex<Vec<Color>>>,
    running: Arc<Mutex<bool>>,
    render_thread: Option<thread::JoinHandle<()>>,
}


impl<T> RayonRenderer<T> where
    T: Environment
{
    pub fn new(screen: [u32;2], env: T) -> RayonRenderer<T> {
        let pixels = Arc::new(Mutex::new(vec![Color::RGB(0xff,0xff,0xff);(screen[0]*screen[1]) as usize]));
        let running = Arc::new(Mutex::new(false));

        let render_thread = None;
        RayonRenderer {screen, env, pixels, running, render_thread}
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

impl<T> Drop for RayonRenderer<T> where
    T: Environment
{
    fn drop(&mut self) {
        self.stop_render();
    }
}

impl<T> Renderer<T> for RayonRenderer<T> where
    T: Environment
{
    fn start_render(&mut self) {
        println!("Start render!");
        self.stop_render();

        {
            let mut running = self.running.lock().unwrap();
            *running = true;
        }

        let a_running = self.running.clone();
        let a_pixels = self.pixels.clone();
        
        let screen = self.screen.clone();
        let env = self.env.clone();
        
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
                let pixel_color = env.render_pixel(j, i, screen);
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
    }

    fn get_pixels(&self) -> Vec<Color> {
        self.pixels.lock().unwrap().clone()
    }

    fn is_ready(&self) -> bool {
        !*self.running.lock().unwrap()
    }
}
