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
    let pos = pos + na::Vector4::new(0.0, 1.01, 0.0, 0.0);

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
