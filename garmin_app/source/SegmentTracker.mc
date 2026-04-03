import Toybox.Lang;
import Toybox.Position;
import Toybox.System;
import Toybox.Math;
import Toybox.Activity;

class SegmentPoint {
    var distance as Float; // distance from start in meters
    var altitude as Float; // altitude in meters
    var elapsedSeconds as Float; // elapsed seconds from start for target pace

    function initialize(dist as Float, alt as Float, seconds as Float) {
        distance = dist;
        altitude = alt;
        elapsedSeconds = seconds;
    }
}

class Segment {
    var name as String;
    var totalDistance as Float;
    var targetTime as Number; // in seconds
    var points as Array<SegmentPoint>;

    function initialize(n as String, dist as Float, time as Number, pts as Array<SegmentPoint>) {
        name = n;
        totalDistance = dist;
        targetTime = time;
        points = pts;
    }
}

class SegmentTracker {
    
    private var mSegments as Array<Segment> = [] as Array<Segment>;
    private var mCurrentSegmentIndex as Number = -1;
    private var mState as Symbol = :idle; // :idle, :approaching, :racing, :results
    private var mStateTime as Number = 0;

    private var mStartDistance as Float = 0.0f;
    private var mStartTime as Number = 0;

    function setSegments(segments as Array<Segment>) as Void {
        mSegments = segments;
        mCurrentSegmentIndex = -1;
        mState = :idle;
    }

    function getState() as Symbol {
        return mState;
    }

    function getActiveSegment() as Segment? {
        if (mCurrentSegmentIndex >= 0 && mCurrentSegmentIndex < mSegments.size()) {
            return mSegments[mCurrentSegmentIndex];
        }
        return null;
    }

    function getNextSegment() as Segment? {
        var nextIdx = mCurrentSegmentIndex + 1;
        if (nextIdx < mSegments.size()) {
            return mSegments[nextIdx];
        }
        return null;
    }

    function update(info as Activity.Info) as Void {
        var nextSeg = getNextSegment();
        
        // 1. Check for segment transitions via Course Points (Primary)
        if (info has :nameOfNextPoint && info.nameOfNextPoint != null && info.distanceToNextPoint != null) {
            var name = info.nameOfNextPoint as String;
            var dist = info.distanceToNextPoint as Float;
            
            if (name.find("[SEG]") == 0) {
                if (mState == :idle || mState == :results) {
                    if (dist < 2000) {
                        mState = :approaching;
                    }
                }
                
                if (name.find("Start") != null && dist < 50) {
                    startSegment();
                }

                if (mState == :racing && name.find("End") != null && dist < 50) {
                    finishSegment();
                }
            }
        }

        // 2. Fallback: GPS Proximity for start (if no course points)
        if (mState == :idle || mState == :approaching || mState == :results) {
            if (nextSeg != null && info.currentLocation != null) {
                var loc = info.currentLocation.toDegrees();
                var points = nextSeg.points;
                if (points.size() > 0) {
                    // This assumes the first point is the start. 
                    // To be fully accurate we'd need lat/lon in SegmentPoint, 
                    // but for this mock/demo we'll rely on course points or simple distance.
                }
            }
        }

        // 3. Racing: Check for completion by distance
        if (mState == :racing) {
            var active = getActiveSegment();
            if (active != null) {
                var relativeDist = (info.elapsedDistance != null ? info.elapsedDistance : 0.0f) - mStartDistance;
                if (relativeDist >= active.totalDistance + 20) { // 20m buffer
                    finishSegment();
                }
            }
        }

        // 4. Results timeout
        if (mState == :results) {
            if (System.getTimer() - mStateTime > 10000) { // 10 seconds
                mState = :idle;
            }
        }
    }

    private function startSegment() as Void {
        if (mState == :racing) { return; }
        mCurrentSegmentIndex++;
        mState = :racing;
        mStartTime = System.getTimer();
        // mStartDistance will be set by the view or activity info
        System.println("STARTING SEGMENT: " + getActiveSegment().name);
    }

    private function finishSegment() as Void {
        if (mState != :racing) { return; }
        mState = :results;
        mStateTime = System.getTimer();
        System.println("FINISHED SEGMENT");
    }

    function setStartDistance(dist as Float) as Void {
        mStartDistance = dist;
    }

    // The rest of the calculation functions...
    function getAheadBehind(totalDistance as Float, totalElapsedSeconds as Number) as Float {
        var seg = getActiveSegment();
        if (seg == null) { return 0.0f; }
        
        var points = seg.points;
        var relativeDistance = totalDistance - mStartDistance;
        if (relativeDistance < 0) { relativeDistance = 0.0f; }

        var targetElapsedSeconds = 0.0f;
        for (var i = 1; i < points.size(); i++) {
            if (relativeDistance <= points[i].distance) {
                var p1 = points[i-1];
                var p2 = points[i];
                var distRatio = (relativeDistance - p1.distance) / (p2.distance - p1.distance);
                targetElapsedSeconds = p1.elapsedSeconds + distRatio * (p2.elapsedSeconds - p1.elapsedSeconds);
                break;
            }
            if (i == points.size() - 1) {
                targetElapsedSeconds = points[i].elapsedSeconds;
            }
        }
        return (totalElapsedSeconds.toFloat() - targetElapsedSeconds);
    }
}
