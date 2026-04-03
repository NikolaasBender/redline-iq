import Toybox.Communications;
import Toybox.System;
import Toybox.Lang;

class CloudSyncer {
    
    // Callback definition
    // function onReceive(responseCode as Number, data as Dictionary?) as Void
    // 

    function fetchSegments(token as String, callback as Method(responseCode as Number, data as Dictionary?) as Void) as Void {
        var url = "http://0.0.0.0:8080/api/segments";
        
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
        if (responseCode == 200) {
            app.setSyncStatus(:success);
        } else {
            app.setSyncStatus(:error);
        }

        if (mCallback != null) {
            mCallback.invoke(responseCode, data);
        }
    }
}
