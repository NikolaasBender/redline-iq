import Toybox.WatchUi;
import Toybox.Application;
import Toybox.Lang;

class LiveSegmentDelegate extends WatchUi.InputDelegate {

    function initialize() {
        InputDelegate.initialize();
    }

    function onTap(clickEvent as WatchUi.ClickEvent) as Boolean {
        var coords = clickEvent.getCoordinates();
        var x = coords[0];
        var width = System.getDeviceSettings().screenWidth;
        var app = Application.getApp() as LiveSegmentApp;
        
        if (x < width / 2) {
            app.triggerSync();
        } else {
            app.toggleMockPhoneConnection();
        }
        return true;
    }

    function onSelect() as Boolean {
        var app = Application.getApp() as LiveSegmentApp;
        app.triggerSync(); // Default to sync for button press
        return true;
    }
}
