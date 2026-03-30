use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    pub name: String,
    pub start_lat: f64,
    pub start_lon: f64,
    pub end_lat: f64,
    pub end_lon: f64,
}

pub fn parse_gpx_segments(gpx_content: &str) -> Vec<Segment> {
    // Currently a simple mock. In full production, we'd use quick-xml to read nodes.
    let mut segments = Vec::new();
    
    if gpx_content.contains("[Segment Start]") {
        segments.push(Segment {
            name: "Parsed Segment from Note".to_string(),
            start_lat: 40.0,
            start_lon: -105.0,
            end_lat: 40.1,
            end_lon: -105.1,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gpx_empty() {
        let content = "<?xml version=\"1.0\"?><gpx></gpx>";
        let segments = parse_gpx_segments(content);
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_parse_gpx_with_segment_notes() {
        let content = r#"<gpx><name>[Segment Start] Climb</name></gpx>"#;
        let segments = parse_gpx_segments(content);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].name, "Parsed Segment from Note");
    }
}
