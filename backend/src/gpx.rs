use serde::{Deserialize, Serialize};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    pub name: String,
    pub start_lat: f64,
    pub start_lon: f64,
    pub end_lat: f64,
    pub end_lon: f64,
    pub start_distance_m: f64,
    pub end_distance_m: f64,
    pub distance_m: f64,
    pub elevation_gain_m: f64,
    pub polyline: Vec<[f64; 2]>,
}

#[derive(Debug, Clone)]
struct Point {
    lat: f64,
    lon: f64,
    ele: f64,
    dist: f64,
}

pub fn parse_gpx(gpx_content: &str) -> (String, Vec<Segment>, Vec<[f64; 2]>) {
    let mut reader = Reader::from_str(gpx_content);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut name = "Uploaded Route".to_string();
    let mut points = Vec::new();
    let mut waypoints = Vec::new();
    
    let mut current_tag = String::new();
    let mut current_lat = 0.0;
    let mut current_lon = 0.0;
    let mut current_ele = 0.0;
    let mut current_wpt_name = String::new();
    let mut in_metadata = false;
    let mut in_trk = false;
    let mut in_wpt = false;
    let mut in_trkpt = false;

    let mut cumulative_dist = 0.0;
    let mut prev_lat: Option<f64> = None;
    let mut prev_lon: Option<f64> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match current_tag.as_str() {
                    "metadata" => in_metadata = true,
                    "trk" => in_trk = true,
                    "wpt" => {
                        in_wpt = true;
                        current_lat = e.attributes().find(|a| a.as_ref().map(|a| a.key.as_ref() == b"lat").unwrap_or(false))
                            .and_then(|a| a.ok()).and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok()).unwrap_or(0.0);
                        current_lon = e.attributes().find(|a| a.as_ref().map(|a| a.key.as_ref() == b"lon").unwrap_or(false))
                            .and_then(|a| a.ok()).and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok()).unwrap_or(0.0);
                        current_wpt_name = String::new();
                    }
                    "trkpt" => {
                        in_trkpt = true;
                        current_lat = e.attributes().find(|a| a.as_ref().map(|a| a.key.as_ref() == b"lat").unwrap_or(false))
                            .and_then(|a| a.ok()).and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok()).unwrap_or(0.0);
                        current_lon = e.attributes().find(|a| a.as_ref().map(|a| a.key.as_ref() == b"lon").unwrap_or(false))
                            .and_then(|a| a.ok()).and_then(|a| String::from_utf8_lossy(&a.value).parse::<f64>().ok()).unwrap_or(0.0);
                        current_ele = 0.0;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if (in_metadata || in_trk) && current_tag == "name" && !in_wpt && !in_trkpt {
                    name = text;
                } else if in_wpt && current_tag == "name" {
                    current_wpt_name = text;
                } else if in_trkpt && current_tag == "ele" {
                    current_ele = text.parse::<f64>().unwrap_or(0.0);
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "metadata" => in_metadata = false,
                    "trk" => in_trk = false,
                    "wpt" => {
                        in_wpt = false;
                        waypoints.push((current_wpt_name.clone(), current_lat, current_lon));
                    }
                    "trkpt" => {
                        in_trkpt = false;
                        if let (Some(p_lat), Some(p_lon)) = (prev_lat, prev_lon) {
                            cumulative_dist += haversine(p_lat, p_lon, current_lat, current_lon);
                        }
                        points.push(Point { lat: current_lat, lon: current_lon, ele: current_ele, dist: cumulative_dist });
                        prev_lat = Some(current_lat);
                        prev_lon = Some(current_lon);
                    }
                    _ => {}
                }
                current_tag = String::new();
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    let mut segments = Vec::new();
    let mut active_start: Option<(String, f64, f64, f64)> = None;

    let waypoints_with_dist: Vec<(String, f64, f64, f64)> = waypoints.into_iter().map(|(name, lat, lon)| {
        let best_pt = points.iter().min_by(|a, b| {
            let da = haversine(a.lat, a.lon, lat, lon);
            let db = haversine(b.lat, b.lon, lat, lon);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        let dist = best_pt.map(|p| p.dist).unwrap_or(0.0);
        (name, lat, lon, dist)
    }).collect();

    for (wp_name, lat, lon, dist) in waypoints_with_dist {
        if wp_name.contains("[Segment Start]") {
            let clean_name = wp_name.replace("[Segment Start]", "").trim().to_string();
            active_start = Some((clean_name, lat, lon, dist));
        } else if wp_name.contains("[Segment End]") {
            if let Some((start_name, s_lat, s_lon, s_dist)) = active_start.take() {
                let name = if start_name.is_empty() { "Unnamed Segment".to_string() } else { start_name };
                
                let segment_points: Vec<Point> = points.iter()
                    .filter(|p| p.dist >= s_dist && p.dist <= dist)
                    .cloned()
                    .collect();
                
                let polyline = segment_points.iter().map(|p| [p.lat, p.lon]).collect();
                let ele_gain = if segment_points.len() > 1 {
                    let start_ele = segment_points.first().unwrap().ele;
                    let end_ele = segment_points.last().unwrap().ele;
                    (end_ele - start_ele).max(0.0)
                } else {
                    0.0
                };

                segments.push(Segment {
                    name,
                    start_lat: s_lat,
                    start_lon: s_lon,
                    end_lat: lat,
                    end_lon: lon,
                    start_distance_m: s_dist,
                    end_distance_m: dist,
                    distance_m: dist - s_dist,
                    elevation_gain_m: ele_gain,
                    polyline,
                });
            }
        }
    }

    let full_polyline = points.iter().map(|p| [p.lat, p.lon]).collect();
    (name, segments, full_polyline)
}

fn haversine(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371000.0;
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let delta_phi = (lat2 - lat1).to_radians();
    let delta_lambda = (lon2 - lon1).to_radians();

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gpx_empty() {
        let content = "<?xml version=\"1.0\"?><gpx></gpx>";
        let (_, segments, _) = parse_gpx(content);
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_parse_gpx_with_segment_notes() {
        let content = r#"<gpx>
            <trk><name>Test Route</name><trkseg>
                <trkpt lat="40.0" lon="-105.0"><ele>1000</ele></trkpt>
                <trkpt lat="40.1" lon="-105.1"><ele>1100</ele></trkpt>
            </trkseg></trk>
            <wpt lat="40.0" lon="-105.0"><name>[Segment Start] Climb</name></wpt>
            <wpt lat="40.1" lon="-105.1"><name>[Segment End]</name></wpt>
        </gpx>"#;
        let (name, segments, polyline) = parse_gpx(content);
        assert_eq!(name, "Test Route");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].name, "Climb");
        assert_eq!(polyline.len(), 2);
        assert!(segments[0].distance_m > 0.0);
        assert_eq!(segments[0].elevation_gain_m, 100.0);
    }
}
