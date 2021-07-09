use clap::{App, load_yaml};

use rust_blackhole::start_windowed;

fn main() {
    // == Deal with CLI arguments ==
    let args_file = load_yaml!("args.yaml");
    let matches = App::from_yaml(args_file).get_matches();

    // Screen
    let scale: u32 = matches.value_of("scale").unwrap_or("1").parse().unwrap();
    
    let screen_raw = matches.value_of("screen").unwrap_or("400");
    let screen: [u32;2] = match screen_raw.parse::<u32>() {
        Ok(screen) => [screen, screen],
        Err(_) => {
            let screen = &screen_raw.split("x").map(|x| {
                x.parse::<u32>().unwrap()
            }).collect::<Vec<u32>>()[..2];
            [screen[0], screen[1]]
        }
    };

    let aspect = screen[0] as f64 / screen[1] as f64;

    // Parameters
    let schwarzschild: bool = matches.is_present("schwarzschild");
    
    let skydome = match matches.value_of("skydome") {
        Some(path) => {
            match image::open(path) {
                Ok(image) => {
                    Some(Box::new(image.into_rgb8()))
                },
                Err(_) => None,
            }
        },
        None => None,
    };

    let r: f64 = matches.value_of("cam-r").unwrap_or("10.0").parse().unwrap();
    let theta: f64 = matches.value_of("cam-theta").unwrap_or("asdf").parse().unwrap_or(std::f64::consts::FRAC_PI_2 - 0.2);
    let phi: f64 = matches.value_of("cam-phi").unwrap_or("0.0").parse().unwrap();

    start_windowed(screen, scale, aspect, schwarzschild, skydome, (r, theta, phi));
}
