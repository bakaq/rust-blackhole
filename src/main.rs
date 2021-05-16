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

    start_windowed(screen, scale, aspect, schwarzschild, skydome);
}
