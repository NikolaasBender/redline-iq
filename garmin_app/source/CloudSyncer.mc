import Toybox.Communications;
import Toybox.System;
import Toybox.Lang;

class CloudSyncer {
    
    // Callback definition
    // function onReceive(responseCode as Number, data as Dictionary?) as Void
    // 

    function fetchSegments(token as String, callback as Method(responseCode as Number, data as Dictionary?) as Void) as Void {
        var url = "http://127.0.0.1:8080/api/segments";
        
        var options = {
            :method => Communications.HTTP_REQUEST_METHOD_GET,
            :headers => {
                "Authorization" => "Bearer " + token
            },
            :responseType => Communications.HTTP_RESPONSE_CONTENT_TYPE_JSON
        };

        var app = Application.getApp() as LiveSegmentApp;
        app.setSyncStatus(:syncing);

        Communications.makeWebRequest(
            url,
            null, // parameters
            options,
            method(:onReceive)
        );
        
        // Internal wrapper to update status
        mCallback = callback;
    }

    private var mCallback as Method(responseCode as Number, data as Dictionary?) as Void?;

    function onReceive(responseCode as Number, data as Dictionary?) as Void {
        var app = Application.getApp() as LiveSegmentApp;
        if (responseCode == 200 && data != null) {
            var segmentsData = data.get("segments") as Array<Dictionary>;
            if (segmentsData != null) {
                var segments = [] as Array<Segment>;
                for (var i = 0; i < segmentsData.size(); i++) {
                    var s = segmentsData[i];
                    // Polyline removed for memory optimization
                    var points = [] as Array<SegmentPoint>;
                    
                    // We need to convert the flat polyline into SegmentPoint objects.
                    // For now, since the backend stores [lat, lon], and SegmentPoint expects [dist, alt, time],
                    // we'll need to do some calculation or assume the backend provides the enriched format soon.
                    // BUT, to keep it simple and working with the current plan:
                    // The backend should return the points array as expected.
                    
                    var ptsData = s.get("points") as Array<Dictionary>?;
                    if (ptsData != null) {
                         for (var j = 0; j < ptsData.size(); j++) {
                            var p = ptsData[j];
                            points.add(new SegmentPoint(
                                p.get("distance").toFloat(),
                                p.get("altitude").toFloat(),
                                p.get("elapsedSeconds").toFloat()
                            ));
                        }
                    }

                    segments.add(new Segment(
                        s.get("name") as String,
                        s.get("distance_m").toFloat(),
                        0, // targetTime (computed from points)
                        points
                    ));
                }
                app.setSegments(segments);
                app.setSyncStatus(:success);
            } else {
                app.setSyncStatus(:error);
            }
        } else {
            app.setSyncStatus(:error);
        }

        if (mCallback != null) {
            mCallback.invoke(responseCode, data);
        }
    }
}
