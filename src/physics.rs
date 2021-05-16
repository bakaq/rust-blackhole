use sdl2::pixels::Color;

use nalgebra as na;
use na::{Vector3, Vector4};


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
    
    match (lambda, mu, nu) {
        (0, 0, 1) => |pos: &Vector4<f64>| {
            1.0/(2.0*pos[1]*(pos[1]-1.0))
        },
        (1, 0, 0) => |pos: &Vector4<f64>|{
            (pos[1] - 1.0)/(2.0*pos[1].powf(3.0))
        },
        (1, 1, 1) => |pos: &Vector4<f64>| {
            -1.0/(2.0*pos[1]*(pos[1]-1.0))
        },
        (1, 2, 2) => |pos: &Vector4<f64>|{
            1.0 - pos[1]
        },
        (1, 3, 3) => |pos: &Vector4<f64>|{
            (1.0 - pos[1])*pos[2].sin().powf(2.0)    
        },
        (2, 1, 2) => inv_r,
        (2, 3, 3) => |pos: &Vector4<f64>|{
            -pos[2].sin()*pos[2].cos()
        },
        (3, 1, 3) => inv_r,
        (3, 2, 3) => |pos: &Vector4<f64>|{
            1.0/pos[2].tan()
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

pub fn cart2sph_at(p: &Vector3<f64>, v: &Vector3<f64>) -> Vector3<f64> {
    let r_hat = Vector3::new(p[0], p[1], p[2]).normalize();
    let phi_hat = Vector3::new(-p[2].sin(), p[2].cos(), 0.0);
    let theta_hat = phi_hat.cross(&r_hat);
    
    Vector3::new(
        v.dot(&r_hat), // r
        v.dot(&theta_hat),
        v.dot(&phi_hat)
    ) 
}

/*
pub fn sph2cart_at(p: &Vector3<f64>, v: &Vector3<f64>) {
    Vector3::new(
        
    )
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    use rand::prelude::*;
    use nalgebra as na;

    #[test]
    fn g_is_digonal() {
        let pos = na::Vector4::<f64>::new(random(), random(), random(), random());
        let pos = pos + na::Vector4::new(0.0, 1.01, 0.0, 0.0);

        for mu in 0..4 {
            for nu in 0..4 {
                if mu == nu {
                    continue;
                }

                assert_eq!(g(mu, nu)(&pos), 0.0);
            }
        }
    }

    #[test]
    fn gamma_is_symetric() {
        let pos = na::Vector4::<f64>::new(random(), random(), random(), random());
        let pos = 100.0 * pos + na::Vector4::new(0.0, 1.01, 0.0, 0.0);

        for lambda in 0..4 {
            for mu in 0..4 {
                for nu in 0..4 {
                    assert_eq!(
                        gamma(lambda, mu, nu)(&pos),
                        gamma(lambda, nu, mu)(&pos), 
                        "Failed at {:?}",
                        (lambda, mu, nu)
                    );
                }
            }
        }
    }

    #[test]
    fn time_norm_makes_proper_time_zero() {
        let pos = na::Vector4::<f64>::new(random(), random(), random(), random());
        let pos = 100.0 * pos + na::Vector4::new(0.0, 1.01, 0.0, 0.0);
        
        let v = na::Vector4::new(0.0, random(), random(), random());
        let mut v = 100.0 * v;

        time_norm(&pos, &mut v);

        let mut s = 0.0;
        for mu in 0..4 {
            for nu in 0..4 {
                s += g(mu, nu)(&pos) * v[mu] * v[nu];
            }
        }
        assert!(s < 0.01);
    }

    #[test]
    fn vec3to4_and_vec4to3_are_inverses() {
        let v3 = na::Vector3::<f64>::new(random(), random(), random());
        let v3_new = vec4to3(&vec3to4(&v3));
        
        for i in 0..3 {
            assert!(v3_new[i] - v3[i] < 0.0001);
        }
        
        let v4 = na::Vector4::<f64>::new(random(), random(), random(), random());
        let v4_new = vec3to4(&vec4to3(&v4));
        
        for i in 0..4 {
            assert!(v4_new[i] - v4[i] < 0.0001);
        }
    }
}
