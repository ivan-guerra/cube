use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'f',
        long,
        default_value_t = 1,
        value_parser = clap::value_parser!(u16).range(1..=10),
        help = "field of view angle in degrees"
    )]
    fov_angle_deg: u16,

    #[arg(
        short = 'c',
        long,
        default_value_t = 256,
        value_parser = clap::value_parser!(u16).range(1..=1_000),
        help = "distance of camera from the center of the cube"
    )]
    camera_dist: u16,
}

fn main() {
    let args = Args::parse();
    let camera_settings = cube::CameraSettings::new(args.fov_angle_deg, args.camera_dist);
    if let Err(e) = cube::run(camera_settings) {
        eprintln!("error: {}", e);
    }
}
