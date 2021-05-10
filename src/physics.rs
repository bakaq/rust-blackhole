use sdl2::pixels::Color;

use nalgebra as na;
use na::{Vector4};

pub fn g(mu: usize, nu: usize) -> impl Fn(&Vector4<f64>) -> f64 {
    if mu != nu {
        |pos: &Vector4<f64>| {
            0.0
        }
    } else {
        [
            |pos: &Vector4<f64>| {
                -(1.0 - 1.0/pos[1])
            },
            |pos: &Vector4<f64>| {
                1.0/(1.0 - 1.0/pos[1])
            },
            |pos: &Vector4<f64>| {
                pos[1].powf(2.0)
            },
            |pos: &Vector4<f64>| {
                pos[1].powf(2.0) * pos[2].sin()
            },
        ][mu]
    }
}

pub fn gamma(mu: usize, nu: usize, lambda: usize) -> impl Fn(&Vector4<f64>) -> f64 {
    // Symetry
    let (mu, nu) = if mu > nu {
        (nu, mu)
    } else {
        (mu, nu)
    };

    let zero = |pos: &Vector4<f64>| {
        0.0
    };

    let inv_r = |pos: &Vector4<f64>| {
        1.0/pos[1]
    };

    let weird = |pos: &Vector4<f64>| {
        -1.0/(2.0*pos[1]*(pos[1]-1.0))
    };

    match (lambda, mu, nu) {
        (0, 0, 1) => weird,
        (1, 0, 0) => |pos: &Vector4<f64>|{
            (pos[1] - 1.0)/(2.0*pos[1].powf(3.0))
        },
        (1, 1, 1) => weird,
        (1, 2, 2) => |pos: &Vector4<f64>|{
            -(pos[1] - 1.0)
        },
        (1, 3, 3) => |pos: &Vector4<f64>|{
            -(pos[1] - 1.0)*pos[2].sin().powf(2.0)    
        },
        (2, 1, 2) => inv_r,
        (2, 3, 3) => |pos: &Vector4<f64>|{
            -pos[2].sin()*pos[2].cos()
        },
        (3, 1, 3) => inv_r,
        (3, 2, 3) => |pos: &Vector4<f64>|{
            pos[2].tan()
        },
        _ => zero,
    }
}

pub fn get_accretion_disk_color((r, _theta, _phi): (f64, f64, f64)) -> Color {
    let temperature = 7e3 * r.powf(-3.0/4.0);
    
    let scale = 1e6;
    let intensity = scale/((29622.4/temperature).exp() - 1.0);

    let temperature = temperature / 100.0;

    let r = if temperature <= 66.0 {
        255.0
    } else {
        let r = temperature - 60.0;
        let r = 329.698727446 * r.powf(-0.1332047592);
        r.clamp(0.0, 255.0)
    };

    let g = if temperature <= 6600.0 {
        let g = temperature;
        let g = 99.4708025861 * g.ln() - 161.1195681661;
        g.clamp(0.0, 255.0)
    } else {
        let g = temperature - 60.0;
        let g = 288.1221695283 * g.powf(-0.0755148492);
        g.clamp(0.0, 255.0)
    };

    let b = if temperature >= 66.0 {
	    255.0
    } else {
        if temperature <= 19.0 {
            0.0
        } else {
            let b = temperature - 10.0;
            let b = 138.5177312231 * b.ln() - 305.0447927307;
            b.clamp(0.0, 255.0)
        }
    };
    
    let (r, g, b) = (
        (r*intensity).clamp(0.0, 255.0),
        (g*intensity).clamp(0.0, 255.0),
        (b*intensity).clamp(0.0, 255.0),
    );
    
    Color::RGB(r as u8, g as u8, b as u8)
}
