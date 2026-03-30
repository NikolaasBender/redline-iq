import Toybox.Communications;
import Toybox.System;
import Toybox.Lang;

class CloudSyncer {
    
    // Callback definition
    // function onReceive(responseCode as Number, data as Dictionary?) as Void
    // 

    function fetchSegments(token as String, callback as Method(responseCode as Number, data as Dictionary?) as Void) as Void {
        var url = "https://your-railway-app.up.railway.app/api/segments";
        
        var options = {
            :method => Communications.HTTP_REQUEST_METHOD_GET,
            :headers => {
                "Authorization" => "Bearer " + token
            },
            :responseType => Communications.HTTP_RESPONSE_CONTENT_TYPE_JSON
        };

        Communications.makeWebRequest(
            url,
            null, // parameters
            options,
            callback
        );
    }
}
