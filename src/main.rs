use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'f',
        long,
        default_value_t = 45,
        value_parser = clap::value_parser!(u16).range(1..=180),
        help = "field of view angle in degrees"
    )]
    fov_angle_deg: u16,

    #[arg(
        short = 'c',
        long,
        default_value_t = 45,
        value_parser = clap::value_parser!(u16).range(1..=1_000),
        help = "distance of camera from the center of the cube"
    )]
    camera_dist: u16,
}

fn main() {
    let _args = Args::parse();
}
