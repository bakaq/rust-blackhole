use sdl2::pixels::Color;

use image;
use image::Pixel;

use nalgebra as na;
use na::Vector3;


pub trait Environment: Clone + Send + Sync + 'static {
    fn raytrace(&self, canvas_pos: (f64,f64)) -> Color;
}

#[derive(Clone)]
pub struct EuclidianRaytracing {
    pos: Vector3<f64>,
    dir: Vector3<f64>,
    up: Vector3<f64>,
    near: f64,
    fovy: f64,
    aspect: f64, // x/y
    skydome: Option<Box<image::RgbImage>>,
}

impl EuclidianRaytracing {
    pub fn new(pos: Vector3<f64>, dir: Vector3<f64>, up:Vector3<f64>, near: f64, fovy: f64, aspect: f64, skydome: Option<Box<image::RgbImage>>) -> EuclidianRaytracing {
        let up = (dir.cross(&up)).cross(&dir).normalize();
        let dir = dir.normalize();
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
    fn raytrace(&self, (canvas_x, canvas_y): (f64,f64)) -> Color {
        // Sphere
        let sphere_pos = Vector3::new(0.0, 0.0, 0.0);
        let r: f64 = 1.0;

        // Find direction
        let ys = (self.fovy/2.0).tan() * self.near;
        let canvas_orig = &self.pos + self.near * &self.dir;
        let dv = &self.up * (canvas_y * ys/2.0) + self.dir.cross(&self.up) * (canvas_x * ys * self.aspect/2.0);
        let pixel_pos = &canvas_orig + &dv;
        let dir = (&pixel_pos - &self.pos).normalize();

        // Check if hit
        let mut hit = false;

        let to_sphere = &sphere_pos - &self.pos;
        let to_closest = to_sphere.dot(&dir) * &dir;
        let closest = &self.pos + &to_closest;
        let r_closest = &closest - &sphere_pos;
        let r_c2 = r_closest.norm_squared();
        if r_c2 < r.powf(2.0) {
            hit = true;
        }

        if hit {
            let hit_point = closest - (r.powf(2.0) - r_c2).sqrt()*&dir;
            let normal = (&hit_point - &sphere_pos).normalize();
            
            let light = Vector3::new(-1.5, 1.5, -1.5).normalize();

            let intensity_raw = -normal.dot(&light);
            let intensity = if intensity_raw < 0.0 {
                0.0
            } else {
                intensity_raw
            }; 

            let int_hex = (intensity * 255.0) as u8;

            Color::RGB(int_hex, int_hex, int_hex)
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
}
