use sdl2::pixels::Color;

use crate::env::Environment;

pub struct Camera<T> where
    T: Environment
{
    env: T,
}

impl<T> Camera<T> where
    T: Environment
{
    pub fn new(env: T) -> Camera<T> {
        Camera{env}
    }


    pub fn render_pixel(&self, x: u32, y: u32, size: [u32; 2]) -> Color {
        let x = x as f64 + 0.5;
        let y = y as f64 + 0.5;

        let sw = size[0] as f64;
        let sh = size[1] as f64;

        let x = (x - sw/2.0)/(sw/2.0);
        let y = (sh/2.0 - y)/(sh/2.0);
       
        self.env.raytrace((x,y))
    }
}

