use std::thread;
use std::time::Duration;

use sdl2::pixels::Color;

use ndarray::prelude::*;
use ndarray_linalg::norm::*;

pub trait Environment {
    fn raytrace(&self, canvas_pos: (f64,f64)) -> Color;
}

pub struct CircleEnv ();

impl CircleEnv {
    #[allow(dead_code)]
    pub fn new() -> CircleEnv {
        CircleEnv()
    }
}

impl Environment for CircleEnv {
    fn raytrace(&self, canvas_pos: (f64,f64)) -> Color {
        let r = (canvas_pos.0.powf(2.0) + canvas_pos.1.powf(2.0)).sqrt();
        
        thread::sleep(Duration::new(0, 1e5 as u32));

        if r < 1.0 {
            Color::RGB(0x00, 0x00, 0x00)
        } else {
            Color::RGB(0xFF, 0xFF, 0xFF)
        }
    }
}

pub struct EuclidianRaytracing {
    pos: Array1<f64>,
    #[allow(dead_code)]
    dir: Array1<f64>,
}

impl EuclidianRaytracing {
    pub fn new(pos: Array1<f64>, dir: Array1<f64>) -> EuclidianRaytracing {
        // TODO: shape check on arrays
        EuclidianRaytracing {pos, dir}
    }
}

impl Environment for EuclidianRaytracing {
    fn raytrace(&self, canvas_pos: (f64,f64)) -> Color {
        // Sphere
        let sphere_pos = array![0.0, 0.0, -3.0];
        let r = 1.0f64;

        // Find direction
        let dir = array![canvas_pos.0/2.0, canvas_pos.1/2.0, -1.0];
        let dir = &dir/dir.norm();

        // Check if hit
        let mut hit = false;

        let to_sphere = &sphere_pos - &self.pos;
        let to_closest = to_sphere.dot(&dir) * &dir;
        let closest = &self.pos + &to_closest;
        let r_closest = &closest - &sphere_pos;
        let r_c2 = r_closest.dot(&r_closest);
        if r_c2 < r.powf(2.0) {
            hit = true;
        }

        if hit {
            let hit_point = closest - (r.powf(2.0) - r_c2).sqrt()*&dir;
            let normal = hit_point - sphere_pos;
            let normal = &normal/normal.norm();
            
            let light: Array1<f64> = array![1.0, -1.5, -1.5];
            let light = &light/light.norm();


            let intensity_raw = -normal.dot(&light);
            let intensity = if intensity_raw < 0.0 {
                0.0
            } else {
                intensity_raw
            }; 

            let int_hex = (intensity * 255.0) as u8;

            Color::RGB(int_hex, int_hex, int_hex)
        } else {
            Color::RGB(0xff, 0xff, 0xff)
        }
    }
}
