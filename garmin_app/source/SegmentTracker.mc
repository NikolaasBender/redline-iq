import Toybox.Position;
import Toybox.System;
import Toybox.Math;

class SegmentTracker {
    
    // The haversine formula to find distance between two lat/lon coordinates in meters
    function getDistance(lat1 as Double, lon1 as Double, lat2 as Double, lon2 as Double) as Float {
        var R = 6371e3; // Earth radius in meters
        var phi1 = lat1 * Math.PI / 180;
        var phi2 = lat2 * Math.PI / 180;
        var dphi = (lat2 - lat1) * Math.PI / 180;
        var dlambda = (lon2 - lon1) * Math.PI / 180;

        var a = Math.sin(dphi/2) * Math.sin(dphi/2) +
                Math.cos(phi1) * Math.cos(phi2) *
                Math.sin(dlambda/2) * Math.sin(dlambda/2);
        
        var c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a));
        return (R * c).toFloat();
    }

    function checkSegmentStart(currentLat as Double, currentLon as Double, targetLat as Double, targetLon as Double) as Boolean {
        // If within 50 meters of the start point, trigger the segment
        var distance = getDistance(currentLat, currentLon, targetLat, targetLon);
        return distance < 50.0;
    }

    function calculateEstimatedRemainingTime(distanceRemaining as Float, currentSpeed as Float) as Float {
        if (currentSpeed > 0.5) {
            return distanceRemaining / currentSpeed; // seconds
        }
        return 0.0;
    }
}
