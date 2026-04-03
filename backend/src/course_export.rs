use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    pub name: String,
    pub polyline: Vec<[f64; 2]>, // [lat, lon]
    pub distance_m: f64,
    pub elevation_gain_m: f64,
}

pub fn generate_gpx_course(name: &str, full_polyline: &[[f64; 2]], segments: &[Segment]) -> String {
    let mut gpx = String::new();
    
    // Header
    gpx.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    gpx.push_str("<gpx version=\"1.1\" creator=\"Redline IQ\" xmlns=\"http://www.topografix.com/GPX/1/1\">\n");
    gpx.push_str(&format!("  <metadata><name>{}</name></metadata>\n", name));

    // 1. Waypoints for Segments (Course Points)
    for seg in segments {
        if let Some(start) = seg.polyline.first() {
            gpx.push_str(&format!("  <wpt lat=\"{:.7}\" lon=\"{:.7}\">\n", start[0], start[1]));
            gpx.push_str(&format!("    <name>[SEG] {} Start</name>\n", seg.name));
            gpx.push_str("    <type>segment_start</type>\n");
            gpx.push_str("  </wpt>\n");
        }
        if let Some(end) = seg.polyline.last() {
            gpx.push_str(&format!("  <wpt lat=\"{:.7}\" lon=\"{:.7}\">\n", end[0], end[1]));
            gpx.push_str(&format!("    <name>[SEG] {} End</name>\n", seg.name));
            gpx.push_str("    <type>segment_end</type>\n");
            gpx.push_str("  </wpt>\n");
        }
    }

    // 2. Main Track
    gpx.push_str("  <trk>\n");
    gpx.push_str(&format!("    <name>{}</name>\n", name));
    gpx.push_str("    <trkseg>\n");
    
    for point in full_polyline {
        gpx.push_str(&format!("      <trkpt lat=\"{:.7}\" lon=\"{:.7}\" />\n", point[0], point[1]));
    }
    
    gpx.push_str("    </trkseg>\n");
    gpx.push_str("  </trk>\n");
    gpx.push_str("</gpx>");

    gpx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpx_generation() {
        let polyline = vec![[40.0, -100.0], [40.1, -100.1]];
        let segments = vec![Segment {
            name: "Test Seg".to_string(),
            polyline: vec![[40.0, -100.0], [40.05, -100.05]],
            distance_m: 1000.0,
            elevation_gain_m: 50.0,
        }];
        
        let gpx = generate_gpx_course("My Route", &polyline, &segments);
        
        assert!(gpx.contains("<name>My Route</name>"));
        assert!(gpx.contains("[SEG] Test Seg Start"));
        assert!(gpx.contains("[SEG] Test Seg End"));
        assert!(gpx.contains("<trkseg>"));
        assert!(gpx.contains("lat=\"40.0000000\""));
    }
}
