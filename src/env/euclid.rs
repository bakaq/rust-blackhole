use sdl2::pixels::Color;

use image;
use image::Pixel;

use nalgebra as na;
use na::{Vector3, Unit};

use crate::physics;

use super::*;


#[derive(Clone)]
pub struct EuclidianRaytracing {
    pos: Vector3<f64>,
    dir: Unit<Vector3<f64>>,
    up: Unit<Vector3<f64>>,
    near: f64,
    fovy: f64,
    aspect: f64, // x/y
    skydome: Option<Box<image::RgbImage>>,
}

impl EuclidianRaytracing {
    pub fn new(pos: Vector3<f64>, dir: Vector3<f64>, up:Vector3<f64>, near: f64, fovy: f64, aspect: f64, skydome: Option<Box<image::RgbImage>>) -> EuclidianRaytracing { let up = Unit::new_normalize((dir.cross(&up)).cross(&dir));
        let dir = Unit::new_normalize(dir);
        EuclidianRaytracing {pos, dir, up, near, fovy, aspect, skydome}
    }

    pub fn new_orbiting(pos: Vector3<f64>, aspect: f64, skydome: Option<Box<image::RgbImage>>) -> EuclidianRaytracing {
        EuclidianRaytracing::new(
            pos,
            -pos,
            *Vector3::z_axis(),
            0.1,
            std::f64::consts::PI/2.0,
            aspect,
            skydome,
        )
    }

    pub fn new_orbiting_spherical((r, theta, phi): (f64, f64, f64), aspect: f64, skydome: Option<Box<image::RgbImage>>) -> EuclidianRaytracing{
        let pos = Vector3::new(
            r * theta.sin() * phi.cos(),
            r * theta.sin() * phi.sin(),
            r * theta.cos(),
        );
        
        EuclidianRaytracing::new_orbiting(pos, aspect, skydome)
    }
}

impl Environment for EuclidianRaytracing {
    fn raytrace(&self, canvas: (f64,f64)) -> Color {
        // Sphere
        let sphere_pos = Vector3::new(0.0, 0.0, 0.0);
        let r: f64 = 1.0;

        // Find direction
        let dir = get_pixel_dir(canvas, self.fovy, self.aspect, &self.dir, &self.up);

        // Check accretion disk
        let mut hit = false;
        let mut thing = 0; // 0 for blackhole, 1 for accretion disk
        let mut depth_buffer = 0.0;
        let mut inter_point = Vector3::new(0.0, 0.0, 0.0);

        if (self.pos.z > 0.0 && dir.z < 0.0)
            || (self.pos.z < 0.0 && dir.z > 0.0)
        {
            let d = -self.pos.dot(Vector3::z_axis().as_ref())/(dir.dot(Vector3::z_axis().as_ref()));
            let intersection = &self.pos + &dir * d;

            let rho = (intersection.x.powf(2.0) + intersection.y.powf(2.0)).sqrt();
            if rho > 3.0 && rho < 5.0 {
                hit = true;
                thing = 1;
                depth_buffer = (&intersection - &self.pos).norm();
                inter_point = intersection;
            }
        }


        // Check blackhole
        let to_sphere = &sphere_pos - &self.pos;
        let to_closest = to_sphere.dot(&dir) * &dir;
        let closest = &self.pos + &to_closest;
        let r_closest = &closest - &sphere_pos; 
        let r_c2 = r_closest.norm_squared();
        if r_c2 < r.powf(2.0) {
            let hit_point = closest - (r.powf(2.0) - r_c2).sqrt()*&dir;
            let to_intersection = &hit_point- &self.pos;
            if !hit || to_intersection.norm() < depth_buffer {
                hit = true;
                thing = 0;
                inter_point = hit_point;
            }
        }

        if hit {
            match thing {
                0 => Color::RGB(0x00, 0x00, 0x00), // Blackhole
                1 => physics::get_accretion_disk_color((inter_point.norm(), 0.0, 0.0)), // Accretion disk
                _ => Color::RGB(0xff, 0xff, 0xff),
            }
            
        } else {
            let mut theta = ((dir.x.powf(2.0) + dir.y.powf(2.0)).sqrt()).atan2(dir.z);
            if theta < 0.0 {
                theta += std::f64::consts::TAU;
            }

            let mut phi = dir.y.atan2(dir.x);
            if phi < 0.0 {
                phi += std::f64::consts::TAU;
            }    
            
            match &self.skydome {
                Some(skydome) => {
                    let (w, h) = skydome.dimensions();
                    
                    let x = (phi / std::f64::consts::TAU * (w as f64)) as u32;
                    let y = (theta / std::f64::consts::PI * (h as f64)) as u32;

                    let pixel = skydome.get_pixel(x, y).channels();
                    Color::RGB(pixel[0], pixel[1], pixel[2])
                },
                None => {
                    if ((phi / std::f64::consts::TAU * 100.0).fract() < 0.25)
                     || ((theta / std::f64::consts::PI * 50.0).fract() < 0.25) {
                        Color::RGB(0xff, 0x00, 0x00)
                    } else {
                        Color::RGB(0x00, 0x00, 0xff)
                    }
                },
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
