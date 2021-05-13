use sdl2::pixels::Color;

use nalgebra::{Vector3, Unit};

use crate::physics::*;

#[cfg(test)]
mod tests;

mod euclid;
pub use euclid::*;

mod schwarzschild;
pub use schwarzschild::*;


pub trait Environment: Clone + Send + Sync + 'static {
    // === Needed ==
    fn raytrace(&self, canvas_pos: (f64,f64)) -> Color;
    
    fn get_data(&self) -> (Vector3<f64>, Unit<Vector3<f64>>, Unit<Vector3<f64>>); // pos, dir, up

    fn set_data(&mut self, pos: &Vector3<f64>, dir: &Vector3<f64>, up: &Vector3<f64>);


    // == Optional ==
    fn pos(&self) -> Vector3<f64> {
        self.get_data().0
    }
    
    fn dir(&self) -> Unit<Vector3<f64>> {
        self.get_data().1
    }
    
    fn up(&self) -> Unit<Vector3<f64>> {
        self.get_data().2
    }


    fn set_pos(&mut self, pos: &Vector3<f64>) {
        self.set_data(&pos, &self.dir(), &self.up())
    }
    
    fn set_dir(&mut self, dir: &Vector3<f64>) {
        self.set_data(&self.pos(), &dir, &self.up())
    }
    
    fn set_up(&mut self, up: &Vector3<f64>) {
        self.set_data(&self.pos(), &self.dir(), &up)
    }
    
    fn set_pos_orbiting(&mut self, pos: &Vector3<f64>) {
        let pos = sph2cart(&pos);
        self.set_pos(&pos);

        let dir = -pos;

        self.set_up(&Vector3::z_axis());
        self.set_dir(&dir);
    }

    fn render_pixel(&self, x: u32, y: u32, screen: [u32; 2]) -> Color {
        let x = x as f64 + 0.5;
        let y = y as f64 + 0.5;

        let sw = screen[0] as f64;
        let sh = screen[1] as f64;

        let x = (x - sw/2.0)/(sw/2.0);
        let y = (sh/2.0 - y)/(sh/2.0);
       
        self.raytrace((x,y))
    }

}
