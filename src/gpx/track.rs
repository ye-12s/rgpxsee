use crate::gpx::segment::Segment;

#[derive(Debug)]
pub struct Track {
    pub segments: Vec<Segment>,
}

impl Track {
    pub fn new(segment: Vec<Segment>) -> Self {
        Self { segments: segment }
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn total_distance_m(&self) -> f64 {
        self.segments.iter().map(|s| s.total_distance_m()).sum()
    }

    pub fn total_ascent_descent_m(&self) -> (f64, f64) {
        let mut ascent = 0.0;
        let mut descent = 0.0;

        for seg in &self.segments {
            let (up, down) = seg.total_ascent_descent_m();
            ascent += up;
            descent += down;
        }

        (ascent, descent)
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}
