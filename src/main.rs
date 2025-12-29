use std::{env, fs::File, io::BufReader, process};

use rgpxsee::gpx::{Track, parse_track};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error :{e}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("Usage: rgpxsee <file.gpx>")?;

    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let track: Track = parse_track(reader)?;

    let distance_km = track.total_distance_m() / 1000.0;
    let (ascent, descent) = track.total_ascent_descent_m();

    // 统计点数
    let point_count: usize = track.segments().iter().map(|s| s.points().len()).sum();

    println!("File: {}", path);
    println!("Segments: {}", track.segment_count());
    println!("Points: {}", point_count);
    println!("Distance: {:.2} km", distance_km);
    println!("Ascent: {:.1} m", ascent);
    println!("Descent: {:.1} m", descent);

    Ok(())
}
