import Toybox.Activity;
import Toybox.Graphics;
import Toybox.Lang;
import Toybox.WatchUi;

class LiveSegmentView extends WatchUi.DataField {

    hidden var mValue as Numeric;

    function initialize() {
        DataField.initialize();
        mValue = 0.0f;
    }

    // The given info object contains all the current workout information.
    // Calculate a value and save it locally in this method.
    // Note that compute() and onUpdate() are asynchronous, and there is no
    // guarantee that compute() will be called before onUpdate().
    function compute(info as Activity.Info) as Void {
        // See Activity.Info in the documentation for available information.
        if(info has :currentSpeed){
            if(info.currentSpeed != null){
                mValue = info.currentSpeed as Numeric;
            } else {
                mValue = 0.0f;
            }
        }
    }

    // Display the value you computed here. This will be called
    // once a second when the data field is visible.
    function onUpdate(dc as Dc) as Void {
        // Clear background
        dc.setColor(Graphics.COLOR_TRANSPARENT, getBackgroundColor());
        dc.clear();
        
        // Set text color contrasting the background
        var textColor = (getBackgroundColor() == Graphics.COLOR_BLACK) ? Graphics.COLOR_WHITE : Graphics.COLOR_BLACK;
        dc.setColor(textColor, Graphics.COLOR_TRANSPARENT);
        
        // Draw the value
        var text = mValue.format("%.2f");
        dc.drawText(
            dc.getWidth() / 2, 
            dc.getHeight() / 2, 
            Graphics.FONT_LARGE, 
            text, 
            Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER
        );
    }

}
