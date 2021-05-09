use sdl2::pixels::Color;

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
