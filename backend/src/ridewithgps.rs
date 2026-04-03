use serde::{Deserialize, Serialize};
use crate::gpx::Segment;
use url::Url;

#[derive(Debug, Deserialize)]
struct RwgpsTrackPoint {
    x: f64, // lon
    y: f64, // lat
    e: Option<f64>, // elevation
    d: Option<f64>, // distance
}

#[derive(Debug, Deserialize)]
struct RwgpsCoursePoint {
    x: f64,
    y: f64,
    d: f64,
    #[serde(default)]
    n: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RwgpsPoi {
    lng: f64,
    lat: f64,
    name: String,
    poi_type_name: String,
}

#[derive(Debug, Deserialize)]
struct RwgpsRouteResponse {
    route: RwgpsRouteData,
}

#[derive(Debug, Deserialize)]
struct RwgpsRouteData {
    name: String,
    track_points: Vec<RwgpsTrackPoint>,
    course_points: Option<Vec<RwgpsCoursePoint>>,
    points_of_interest: Option<Vec<RwgpsPoi>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullRoute {
    pub id: String,
    pub name: String,
    pub full_polyline: Vec<[f64; 2]>,
    pub segments: Vec<Segment>,
}

pub fn extract_id_from_url(url_str: &str) -> Option<String> {
    let url = Url::parse(url_str).ok()?;
    let path_segments: Vec<_> = url.path_segments()?.collect();
    if path_segments.len() >= 2 && path_segments[0] == "routes" {
        return Some(path_segments[1].to_string());
    }
    None
}

/// Find the distance along the route for a given lat/lon by finding the nearest track point.
fn find_distance_for_point(track_points: &[RwgpsTrackPoint], lat: f64, lon: f64) -> f64 {
    track_points.iter()
        .min_by(|a, b| {
            let da = (a.y - lat).powi(2) + (a.x - lon).powi(2);
            let db = (b.y - lat).powi(2) + (b.x - lon).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|tp| tp.d)
        .unwrap_or(0.0)
}

/// Build a Segment from start/end coordinates, distances, and track point data.
fn build_segment(
    name: String,
    start_lat: f64, start_lon: f64, start_d: f64,
    end_lat: f64, end_lon: f64, end_d: f64,
    track_points: &[RwgpsTrackPoint],
) -> Segment {
    let segment_polyline: Vec<[f64; 2]> = track_points.iter()
        .filter(|tp| {
            let dist = tp.d.unwrap_or(0.0);
            dist >= start_d && dist <= end_d
        })
        .map(|tp| [tp.y, tp.x])
        .collect();

    let start_ele = track_points.iter()
        .find(|tp| tp.d.unwrap_or(0.0) >= start_d)
        .and_then(|tp| tp.e)
        .unwrap_or(0.0);
    let end_ele = track_points.iter()
        .find(|tp| tp.d.unwrap_or(0.0) >= end_d)
        .and_then(|tp| tp.e)
        .unwrap_or(0.0);

    Segment {
        name,
        start_lat,
        start_lon,
        end_lat,
        end_lon,
        start_distance_m: start_d,
        end_distance_m: end_d,
        distance_m: end_d - start_d,
        elevation_gain_m: (end_ele - start_ele).max(0.0),
        polyline: segment_polyline,
    }
}

pub async fn fetch_rwgps_route(route_id: &str) -> Result<FullRoute, String> {
    let client = reqwest::Client::new();
    let url = format!("https://ridewithgps.com/routes/{}.json", route_id);
    
    let resp = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !resp.status().is_success() {
        return Err(format!("RWGPS API returned status: {}", resp.status()));
    }

    let body = resp.text()
        .await
        .map_err(|e| format!("Failed to read RWGPS response: {}", e))?;

    // Try wrapped format first ({route: {...}}), fall back to flat format ({...})
    let route_data: RwgpsRouteData = if let Ok(wrapped) = serde_json::from_str::<RwgpsRouteResponse>(&body) {
        wrapped.route
    } else {
        serde_json::from_str::<RwgpsRouteData>(&body)
            .map_err(|e| format!("Failed to parse RWGPS JSON: {}", e))?
    };

    let full_polyline: Vec<[f64; 2]> = route_data.track_points.iter()
        .map(|p| [p.y, p.x])
        .collect();

    let mut segments = Vec::new();

    // Strategy 1: Parse points_of_interest with poi_type_name "segment_start"/"segment_end"
    if let Some(pois) = &route_data.points_of_interest {
        let starts: Vec<&RwgpsPoi> = pois.iter()
            .filter(|p| p.poi_type_name == "segment_start")
            .collect();
        let ends: Vec<&RwgpsPoi> = pois.iter()
            .filter(|p| p.poi_type_name == "segment_end")
            .collect();

        for start_poi in &starts {
            let start_d = find_distance_for_point(&route_data.track_points, start_poi.lat, start_poi.lng);

            // Find the nearest end POI that comes after this start along the route
            let best_end = ends.iter()
                .filter_map(|end_poi| {
                    let end_d = find_distance_for_point(&route_data.track_points, end_poi.lat, end_poi.lng);
                    if end_d > start_d { Some((end_poi, end_d)) } else { None }
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((end_poi, end_d)) = best_end {
                // Clean up common prefixes like "TSS: " from name
                let raw_name = &start_poi.name;
                let name = raw_name
                    .trim_start_matches("TSS:")
                    .trim_start_matches("TSS: ")
                    .trim_start_matches("[Segment Start]")
                    .trim()
                    .to_string();
                let name = if name.is_empty() { "Unnamed Segment".to_string() } else { name };

                segments.push(build_segment(
                    name,
                    start_poi.lat, start_poi.lng, start_d,
                    end_poi.lat, end_poi.lng, end_d,
                    &route_data.track_points,
                ));
            }
        }
    }

    // Strategy 2: Fallback to course_points with [Segment Start]/[Segment End] text markers
    if segments.is_empty() {
        if let Some(course_points) = &route_data.course_points {
            let mut active_start: Option<&RwgpsCoursePoint> = None;

            for cp in course_points {
                let cp_name = cp.n.as_deref().unwrap_or("");
                if cp_name.contains("[Segment Start]") {
                    active_start = Some(cp);
                } else if cp_name.contains("[Segment End]") {
                    if let Some(start) = active_start.take() {
                        let start_name = start.n.as_deref().unwrap_or("");
                        let segment_name = start_name.replace("[Segment Start]", "").trim().to_string();
                        let name = if segment_name.is_empty() { "Unnamed Segment".to_string() } else { segment_name };

                        segments.push(build_segment(
                            name,
                            start.y, start.x, start.d,
                            cp.y, cp.x, cp.d,
                            &route_data.track_points,
                        ));
                    }
                }
            }
        }
    }

    println!("Parsed {} segments from RWGPS route '{}'", segments.len(), route_data.name);

    Ok(FullRoute {
        id: route_id.to_string(),
        name: route_data.name,
        full_polyline,
        segments,
    })
}
