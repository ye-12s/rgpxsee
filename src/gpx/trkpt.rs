use crate::gpx::{Error, Segment, Track, err::InternalError};
use std::io::BufRead;

use quick_xml::{
    Reader,
    events::{BytesStart, BytesText, Event},
};

#[derive(Debug, PartialEq)]
pub struct TrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub time: Option<String>,
    pub ele: Option<f64>,
}

type Applyfn = fn(&mut TrackPoint, &str) -> Result<(), InternalError>;

struct TextHandler {
    tag: &'static [u8],
    apply: Applyfn,
}

fn apply_ele(pt: &mut TrackPoint, s: &str) -> Result<(), InternalError> {
    let v = s
        .parse::<f64>()
        .map_err(|_| InternalError::InvalidTrackPoint("ele is not a number".into()))?;
    pt.ele = Some(v);
    Ok(())
}

fn apply_time(pt: &mut TrackPoint, s: &str) -> Result<(), InternalError> {
    pt.time = Some(s.to_string());
    Ok(())
}

const HANDLERS: &[TextHandler] = &[
    TextHandler {
        tag: b"time",
        apply: apply_time,
    },
    TextHandler {
        tag: b"ele",
        apply: apply_ele,
    },
];

pub fn parse_track<R: BufRead>(reader: R) -> Result<Track, Error> {
    let mut xml = Reader::from_reader(reader);
    xml.trim_text(true);

    let mut buf = Vec::new();
    let mut segments: Vec<Segment> = Vec::new();
    let mut current_points: Vec<TrackPoint> = Vec::new();
    let mut current_handler: Option<Applyfn> = None;
    let mut current_point: Option<TrackPoint> = None;

    loop {
        match xml.read_event_into(&mut buf).map_err(InternalError::from)? {
            Event::Start(e) if e.name().as_ref() == b"trkseg" => {
                current_points.clear();
            }

            Event::End(e) if e.name().as_ref() == b"trkseg" => {
                if !current_points.is_empty() {
                    segments.push(Segment::new(std::mem::take(&mut current_points)));
                }
            }

            Event::Start(e) if e.name().as_ref() == b"trkpt" => {
                current_point = Some(parse_trkpt(&e)?);
                current_handler = None;
            }

            Event::End(e) if e.name().as_ref() == b"trkpt" => {
                if let Some(pt) = current_point.take() {
                    current_points.push(pt);
                }
                current_handler = None;
            }

            Event::Start(e) => {
                if current_point.is_some() {
                    current_handler = find_handler(e.name().as_ref());
                }
            }

            Event::Text(e) => {
                if let (Some(ref mut pt), Some(apply)) = (current_point.as_mut(), current_handler) {
                    let s = read_text_string(e)?;
                    apply(pt, &s)?;
                }
            }

            Event::End(_) => {
                current_handler = None;
            }

            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(Track::new(segments))
}

pub fn parse_track_points<R: BufRead>(reader: R) -> Result<Vec<TrackPoint>, Error> {
    let mut xml = Reader::from_reader(reader);
    xml.trim_text(true);

    let mut buf = Vec::new();
    let mut points = Vec::new();
    let mut current: Option<TrackPoint> = None;
    let mut current_handler: Option<Applyfn> = None;

    loop {
        match xml.read_event_into(&mut buf).map_err(InternalError::from)? {
            Event::Start(e) if e.name().as_ref() == b"trkpt" => {
                current = Some(parse_trkpt(&e)?);
                current_handler = None;
            }

            Event::Start(e) => {
                current_handler = if current.is_some() {
                    find_handler(e.name().as_ref())
                } else {
                    None
                };
            }

            Event::Text(e) => {
                if let (Some(ref mut pt), Some(apply)) = (current.as_mut(), current_handler) {
                    let s = read_text_string(e)?;
                    apply(pt, &s)?;
                }
            }

            Event::End(e) if e.name().as_ref() == b"trkpt" => {
                if let Some(pt) = current.take() {
                    points.push(pt);
                }
            }

            Event::End(_) => {
                current_handler = None;
            }

            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(points)
}

fn find_handler(tag: &[u8]) -> Option<Applyfn> {
    HANDLERS.iter().find(|h| h.tag == tag).map(|h| h.apply)
}

fn read_text_string(e: BytesText) -> Result<String, InternalError> {
    Ok(e.unescape().map_err(InternalError::from)?.to_string())
}

fn parse_attr_f64(
    attr: &quick_xml::events::attributes::Attribute,
    name: &'static str,
) -> Result<f64, InternalError> {
    let value = std::str::from_utf8(&attr.value)
        .map_err(|_| InternalError::InvalidTrackPoint("lat is not valid utf8.".into()))?;
    value
        .parse::<f64>()
        .map_err(|_| InternalError::InvalidTrackPoint(format!("{name} is not a number")))
}

pub fn parse_trkpt(e: &BytesStart) -> Result<TrackPoint, InternalError> {
    let mut lat = None;
    let mut lon = None;
    for attr in e.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"lat" => lat = Some(parse_attr_f64(&attr, "lat")?),
            b"lon" => lon = Some(parse_attr_f64(&attr, "lon")?),
            _ => {}
        }
    }

    match (lat, lon) {
        (Some(lat), Some(lon)) => Ok(TrackPoint {
            lat,
            lon,
            time: None,
            ele: None,
        }),
        _ => Err(InternalError::InvalidTrackPoint(
            "trkpt missing lat or lon.".into(),
        )),
    }
}

#[test]
fn parse_multiple_trkseg() {
    let gpx = r#"
    <gpx>
      <trk>
        <trkseg>
          <trkpt lat="0.0" lon="0.0"><ele>100</ele></trkpt>
          <trkpt lat="0.0" lon="0.001"><ele>110</ele></trkpt>
        </trkseg>
        <trkseg>
          <trkpt lat="0.0" lon="0.001"><ele>110</ele></trkpt>
          <trkpt lat="0.0" lon="0.002"><ele>105</ele></trkpt>
        </trkseg>
      </trk>
    </gpx>
    "#;

    let track = parse_track(std::io::Cursor::new(gpx)).unwrap();

    assert_eq!(track.segment_count(), 2);

    let (up, down) = track.total_ascent_descent_m();
    assert_eq!(up, 10.0);
    assert_eq!(down, 5.0);
}

#[test]
fn parse_single_trkpt() {
    let gpx = r#"
                    <gpx>
                        <trk>
                        <trkseg>
                            <trkpt lat="1.0" lon="2.0">
                                <time>2024-01-01T00:00:00Z</time>
                                <ele>123.45</ele>
                            </trkpt>
                        </trkseg>
                        </trk>
                    </gpx>
                    "#;

    let reader = std::io::Cursor::new(gpx);
    let points = parse_track_points(reader).unwrap();

    assert_eq!(points.len(), 1);

    assert_eq!(points[0].lat, 1.0);
    assert_eq!(points[0].lon, 2.0);
    assert_eq!(points[0].time.as_deref(), Some("2024-01-01T00:00:00Z"));
    assert_eq!(points[0].ele, Some(123.45));
}
