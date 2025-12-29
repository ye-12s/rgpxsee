use crate::gpx::trkpt;

const EARTH_RADIUS_M: f64 = 6_371_000.0;

#[derive(Debug)]
pub struct Segment {
    points: Vec<trkpt::TrackPoint>,
}

impl Segment {
    pub fn new(points: Vec<trkpt::TrackPoint>) -> Self {
        Self { points }
    }

    pub fn points(&self) -> &[trkpt::TrackPoint] {
        &self.points
    }

    pub fn total_distance_m(&self) -> f64 {
        self.points
            .windows(2)
            .map(|w| haversine_m(&w[0], &w[1]))
            .sum()
    }

    pub fn total_ascent_descent_m(&self) -> (f64, f64) {
        let mut ascent = 0.0;
        let mut descent = 0.0;

        for w in self.points.windows(2) {
            let a = &w[0];
            let b = &w[1];

            let (Some(e1), Some(e2)) = (a.ele, b.ele) else {
                continue;
            };

            let delta = e2 - e1;
            if delta > 0.0 {
                ascent += delta;
            } else if delta < 0.0 {
                descent += -delta;
            }
        }
        (ascent, descent)
    }
}

fn haversine_m(pa: &trkpt::TrackPoint, pb: &trkpt::TrackPoint) -> f64 {
    let dlat = (pb.lat - pa.lat).to_radians();
    let dlon = (pb.lon - pa.lon).to_radians();

    let lat1 = pa.lat.to_radians();
    let lat2 = pb.lat.to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);

    let c = 2.0 * h.sqrt().atan2((1.0 - h).sqrt());
    EARTH_RADIUS_M * c
}

#[test]
fn segment_distance_basic() {
    use super::trkpt::TrackPoint;

    let pts = vec![
        TrackPoint {
            lat: 0.0,
            lon: 0.0,
            time: None,
            ele: None,
        },
        TrackPoint {
            lat: 0.0,
            lon: 0.001, // ~111m
            time: None,
            ele: None,
        },
    ];

    let seg = Segment::new(pts);
    let d = seg.total_distance_m();

    assert!(d > 100.0 && d < 120.0);
}

#[test]
fn segment_ascent_descent_basic() {
    use super::trkpt::TrackPoint;

    let pts = vec![
        TrackPoint {
            lat: 0.0,
            lon: 0.0,
            ele: Some(100.0),
            time: None,
        },
        TrackPoint {
            lat: 0.0,
            lon: 0.0,
            ele: Some(120.0),
            time: None,
        },
        TrackPoint {
            lat: 0.0,
            lon: 0.0,
            ele: Some(110.0),
            time: None,
        },
    ];

    let seg = Segment::new(pts);
    let (up, down) = seg.total_ascent_descent_m();

    assert_eq!(up, 20.0);
    assert_eq!(down, 10.0);
}

#[test]
fn segment_ascent_descent_with_missing_ele() {
    use super::trkpt::TrackPoint;

    let pts = vec![
        TrackPoint {
            lat: 0.0,
            lon: 0.0,
            ele: Some(100.0),
            time: None,
        },
        TrackPoint {
            lat: 0.0,
            lon: 0.0,
            ele: None,
            time: None,
        },
        TrackPoint {
            lat: 0.0,
            lon: 0.0,
            ele: Some(130.0),
            time: None,
        },
    ];

    let seg = Segment::new(pts);
    let (up, down) = seg.total_ascent_descent_m();

    assert_eq!(up, 0.0);
    assert_eq!(down, 0.0);
}
