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
        // Set the background color
        View.findDrawableById("Background").setColor(getBackgroundColor());

        // Call parent's onUpdate(dc) to redraw the layout
        View.onUpdate(dc);

        var valueDrawable = View.findDrawableById("value");
        if (valueDrawable != null) {
            valueDrawable.setText(mValue.format("%.2f"));
        }
    }

}
