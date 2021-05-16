use sdl2::pixels::Color;

use nalgebra as na;
use na::{Vector3, Vector4, Unit};

use crate::physics;
use physics::*;

use super::*;

#[derive(Clone)]
pub struct SchwarzschildRaytracing {
    pos: Vector3<f64>,
    dir: Unit<Vector3<f64>>,
    up: Unit<Vector3<f64>>,
    near: f64,
    fovy: f64,
    aspect: f64, // x/y
    skydome: Option<Box<image::RgbImage>>,
}

impl SchwarzschildRaytracing {
    pub fn new(pos: Vector3<f64>, dir: Vector3<f64>, up:Vector3<f64>, near: f64, fovy: f64, aspect: f64, skydome: Option<Box<image::RgbImage>>) -> SchwarzschildRaytracing { let up = Unit::new_normalize((dir.cross(&up)).cross(&dir));
        let dir = Unit::new_normalize(dir);
        SchwarzschildRaytracing {pos, dir, up, near, fovy, aspect, skydome}
    }

    pub fn new_orbiting(pos: Vector3<f64>, aspect: f64, skydome: Option<Box<image::RgbImage>>) -> SchwarzschildRaytracing {
        SchwarzschildRaytracing::new(
            pos,
            -pos,
            *Vector3::z_axis(),
            0.1,
            std::f64::consts::PI/2.0,
            aspect,
            skydome,
        )
    }

    pub fn new_orbiting_spherical((r, theta, phi): (f64, f64, f64), aspect: f64, skydome: Option<Box<image::RgbImage>>) -> SchwarzschildRaytracing{
        let pos = Vector3::new(
            r * theta.sin() * phi.cos(),
            r * theta.sin() * phi.sin(),
            r * theta.cos(),
        );
        
        SchwarzschildRaytracing::new_orbiting(pos, aspect, skydome)
    }
}

impl Environment for SchwarzschildRaytracing {
    fn raytrace(&self, (canvas_x, canvas_y): (f64,f64)) -> Color {
        // Find direction
        let ys = (self.fovy/2.0).tan() * self.near;
        let canvas_orig = &self.pos + self.near * self.dir.as_ref();
        let dv = self.up.as_ref() * (canvas_y * ys/2.0) + self.dir.cross(self.up.as_ref()) * (canvas_x * ys * self.aspect/2.0);
        let pixel_pos = &canvas_orig + &dv;
        let dir = (&pixel_pos - &self.pos).normalize();
    
        // Convert coords
        let mut pos = vec3to4(&cart2sph(&self.pos));

        let mut dir = vec3to4(&cart2sph_at(&vec4to3(&pos), &dir));

        time_norm(&pos, &mut dir);

        println!("dir: {}", vec4to3(&dir));

        // Integrate
        let mut _last_pos = pos;
        let mut last_dir = dir;
        last_dir[1] *= -1.0;

        // This function is wrong
        // TODO: rewrite
        fn vec_diff(v1: Vector4<f64>, v2: Vector4<f64>) -> f64 {
            let v1 = sph2cart(&vec4to3(&v1));
            let v2 = sph2cart(&vec4to3(&v2));

            (v2 - v1).norm()
        }

        let dt = 0.001;
        loop {
            for lambda in 0..4 {
                if pos[lambda].is_nan() || dir[lambda].is_nan() {
                    return Color::RGB(0xff, 0x00, 0x00)
                }
            }

            // Out to infinity
            if dir[1] > 0.0 && vec_diff(dir, last_dir) > 0.1 {
                // TODO: skydome
                //println!("pos: {}, dir: {}", pos, dir);
                break Color::RGB(0x00, 0xff, 0xff)
            }
            // Event horizon
            if pos[1] < 1.01 {
                break Color::RGB(0x00, 0x00, 0x00)
            }

                
            // Update dir
            last_dir = dir;
            for lambda in 0..4 {
                for mu in 0..4 {
                    for nu in 0..4 {
                        dir[lambda] -= gamma(lambda, mu, nu)(&pos)*dir[mu]*dir[nu]*dt
                    }
                }
            }

            if g(0,0)(&pos).is_nan() {
                return Color::RGB(0x00, 0x00, 0xff); 
            }

            if g(0,0)(&pos).abs() < 0.00001 {
                return Color::RGB(0xff, 0x00, 0xff);
            }
            time_norm(&pos ,&mut dir);

            // Update pos
            _last_pos = pos;
            for lambda in 0..4 {
                pos[lambda] += dir[lambda]*dt;
            }
        }
    }

    fn get_data(&self) -> (Vector3<f64>, Unit<Vector3<f64>>, Unit<Vector3<f64>>){
        (self.pos, self.dir, self.up)
    }

    fn set_data(&mut self, pos: &Vector3<f64>, dir: &Vector3<f64>, up: &Vector3<f64>) {
        self.pos = *pos;
        self.dir = Unit::new_normalize(*dir);
        self.up = Unit::new_normalize(dir.cross(&up).cross(&dir));
    }
}
