use std::thread;
use std::time::{Duration};


use image;
use image::RgbImage;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use nalgebra::{Unit, Vector3};


mod render;
mod env;
mod physics;

use render::Renderer;
use env::{EuclidianRaytracing, SchwarzschildRaytracing, Environment};

pub fn start_windowed(screen: [u32;2], scale: u32, aspect: f64, schwarzschild: bool, skydome: Option<Box<image::RgbImage>>, (r, theta, phi): (f64, f64, f64)) {
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

    // Renderer
    /*
    let r = 10.0;
    let theta = std::f64::consts::FRAC_PI_2 - 0.2;
    let phi = 0.0; 
    */

    let mut renderer = render::RayonRenderer::new(screen,
        if schwarzschild {
            Env::Schwarz(SchwarzschildRaytracing::new_orbiting_spherical(
                (r, theta, phi), aspect, skydome.clone()))
        } else {
            Env::Euclid(EuclidianRaytracing::new_orbiting_spherical(
                (r, theta, phi), aspect, skydome.clone()))
        },
    );
    
    renderer.start_render();

    // Mouse state
    let mut last_mouse_pos: Option<(i32, i32)> = None;

    // Main loop
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        // Events
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => (),
            }
        }
        
        
        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x10));
        canvas.clear();
        
        // Rendering
        let pixels = renderer.get_pixels();
        for ii in 0..pixels.len() {
            let ii = ii as u32;
            let i = (ii / screen[0]) * scale;
            let j = (ii % screen[0]) * scale;
            
            canvas.set_draw_color(pixels[ii as usize]);
            canvas.fill_rect(Rect::new(j as i32, i as i32, scale, scale)).unwrap();
        }
   
        // Mouse orbiting
        if event_pump.mouse_state().left() {
            match last_mouse_pos {
                Some(last_pos) => {
                    let pos = (event_pump.mouse_state().x(), event_pump.mouse_state().y());

                    let (dx, dy) =  (pos.0 - last_pos.0, pos.1 - last_pos.1);

                    last_mouse_pos = Some(pos);

                    if (dx, dy) != (0, 0) {
                        let pos = renderer.env.pos();

                        let mut pos = (
                            pos.norm(),
                            (pos.x.powf(2.0) + pos.y.powf(2.0)).sqrt().atan2(pos.z),
                            pos.y.atan2(pos.x),
                        );
                        
                        if pos.1 < 0.0 {
                            pos.1 += std::f64::consts::TAU;
                        }
                        
                        if pos.2 < 0.0 {
                            pos.2 += std::f64::consts::TAU;
                        }

                        let new_pos = Vector3::new(
                            pos.0,
                            pos.1 - (dy as f64)/(std::f64::consts::PI*100.0),
                            pos.2 - (dx as f64)/(std::f64::consts::TAU*100.0),
                        );
                        
                        renderer.env.set_pos_orbiting(&new_pos);
                        renderer.start_render();
                    }
                },
                None => {
                    last_mouse_pos = Some((event_pump.mouse_state().x(), event_pump.mouse_state().y()));
                },
            }
        } else {
            last_mouse_pos = None;
        }

        canvas.present();
        thread::sleep(Duration::new(0, (1e9 as u32) / 60));
    }
}

pub fn render_image(screen: [u32;2], aspect: f64, schwarzschild: bool, skydome: Option<Box<image::RgbImage>>, (r, theta, phi): (f64, f64, f64), path: &str) {
    let mut renderer = render::RayonRenderer::new(screen,
        if schwarzschild {
            Env::Schwarz(SchwarzschildRaytracing::new_orbiting_spherical(
                (r, theta, phi), aspect, skydome.clone()))
        } else {
            Env::Euclid(EuclidianRaytracing::new_orbiting_spherical(
                (r, theta, phi), aspect, skydome.clone()))
        },
    );
    
    renderer.start_render();

    // Creates an image
    let mut img = RgbImage::new(screen[0], screen[1]);
    
    // Rendering
    while !renderer.is_ready() {
        thread::sleep(Duration::from_millis(10));
    }

    let pixels = renderer.get_pixels();

    for ii in 0..pixels.len() {
        let ii = ii as u32;
        let i = ii / screen[0];
        let j = ii % screen[0];
        
        //canvas.set_draw_color(pixels[ii as usize]);
        let pixel = pixels[ii as usize];
        let pixel: image::Rgb<u8> = [pixel.r, pixel.g, pixel.b].into();

        img[(j, i)] = pixel;
        //canvas.fill_rect(Rect::new(j as i32, i as i32, scale, scale)).unwrap();
    }

    img.save(path).unwrap();
    println!("Written image")
}

#[derive(Clone)]
enum Env {
    Euclid(EuclidianRaytracing),
    Schwarz(SchwarzschildRaytracing),
}

impl Environment for Env {
    fn raytrace(&self, coords: (f64, f64)) -> Color {
        match self {
            Self::Euclid(euclid) => euclid.raytrace(coords),
            Self::Schwarz(schwarz) => schwarz.raytrace(coords),
        }
    }
    
    fn get_data(&self) -> (Vector3<f64>, Unit<Vector3<f64>>, Unit<Vector3<f64>>){
        match self {
            Self::Euclid(a) => a.get_data(),
            Self::Schwarz(a) => a.get_data(),
        }
    }

    fn set_data(&mut self, pos: &Vector3<f64>, dir: &Vector3<f64>, up: &Vector3<f64>) {
        match self{
            Self::Euclid(a) => a.set_data(&pos, &dir, &up),
            Self::Schwarz(a) => a.set_data(&pos, &dir, &up),
        }
    }
}
