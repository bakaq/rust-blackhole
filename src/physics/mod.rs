use sdl2::pixels::Color;

use nalgebra as na;
use na::{Vector3, Vector4};

#[cfg(test)]
mod tests;

pub fn g(mu: usize, nu: usize) -> impl Fn(&Vector4<f64>) -> f64 {
    if mu != nu {
        |_pos: &Vector4<f64>| {
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

pub fn gamma(lambda: usize, mu: usize, nu: usize) -> impl Fn(&Vector4<f64>) -> f64 {
    // Symetry
    let (mu, nu) = if mu > nu {
        (nu, mu)
    } else {
        (mu, nu)
    };

    let zero = |_pos: &Vector4<f64>| {
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

pub fn time_norm(p: &Vector4<f64>, v: &mut Vector4<f64>) {
    let v3 = Vector3::new(v[1], v[2], v[3]).normalize();
    let v3_norm = (
        v[0].powf(2.0) +
        (v[1]*p[1]).powf(2.0) +
        (v[2]*p[1]*p[2].sin()).powf(2.0)
    ).sqrt();
    let v3 = v3/v3_norm;

    v[1] = v3[0];
    v[2] = v3[1];
    v[3] = v3[2];

    v[0] = ((
        g(1,1)(&p)*v[1].powf(2.0) + 
        g(2,2)(&p)*v[2].powf(2.0) + 
        g(3,3)(&p)*v[3].powf(2.0)
    )/(-g(0,0)(&p))).sqrt();
}

pub fn vec4to3(v: &Vector4<f64>) -> Vector3<f64> {
    Vector3::new(v[1], v[2], v[3])
}

pub fn vec3to4(v: &Vector3<f64>) -> Vector4<f64> {
    Vector4::new(0.0, v[0], v[1], v[2])
}

pub fn cart2sph(v: &Vector3<f64>) -> Vector3<f64> {
    let mut v = Vector3::new(
        v.norm(),
        (v.x.powf(2.0) + v.y.powf(2.0)).sqrt().atan2(v.z),
        v.y.atan2(v.x),
    );

    if v[1] < 0.0 {
        v[1] += std::f64::consts::TAU;
    }
    
    if v[2] < 0.0 {
        v[1] += std::f64::consts::TAU;
    }
    
    v
}

pub fn sph2cart(v: &Vector3<f64>) -> Vector3<f64> {
    Vector3::new(
       v[0] * v[1].sin() * v[2].cos(),
       v[0] * v[1].sin() * v[2].sin(),
       v[0] * v[1].cos(),
    )
}
