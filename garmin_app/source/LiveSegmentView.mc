import Toybox.Activity;
import Toybox.Graphics;
import Toybox.Lang;
import Toybox.WatchUi;
import Toybox.Time;

class LiveSegmentView extends WatchUi.DataField {

    private var mElapsedSeconds as Number = 0;
    private var mDistance as Float = 0.0f;
    private var mSpeed as Float = 0.0f;

    function initialize() {
        DataField.initialize();
    }

    function compute(info as Activity.Info) as Void {
        if (info.elapsedTime != null) {
            mElapsedSeconds = info.elapsedTime / 1000;
        }
        if (info.elapsedDistance != null) {
            mDistance = info.elapsedDistance;
        }
        if (info.currentSpeed != null) {
            mSpeed = info.currentSpeed;
        }
    }

    function onUpdate(dc as Dc) as Void {
        var bgColor = getBackgroundColor();
        dc.setColor(Graphics.COLOR_TRANSPARENT, bgColor);
        dc.clear();

        var app = Application.getApp() as LiveSegmentApp;
        var tracker = app.getSegmentTracker();
        var segment = tracker.getActiveSegment();

        if (segment == null) {
            drawNoSegment(dc);
            return;
        }

        var aheadBehind = tracker.getAheadBehind(mDistance, mElapsedSeconds);
        
        drawTopBar(dc, segment);
        drawAheadBehind(dc, aheadBehind);
        drawProgressBar(dc, segment, mDistance);
        drawElevationProfile(dc, segment, mDistance);
        drawBottomSection(dc, segment, mDistance, mSpeed);
        
        // Draw Status Icons (Sync/BT) in corners
        drawStatusIcons(dc);
    }

    private function drawNoSegment(dc as Dc) as Void {
        var textColor = (getBackgroundColor() == Graphics.COLOR_BLACK) ? Graphics.COLOR_WHITE : Graphics.COLOR_BLACK;
        dc.setColor(textColor, Graphics.COLOR_TRANSPARENT);
        dc.drawText(dc.getWidth() / 2, dc.getHeight() / 2, Graphics.FONT_MEDIUM, "Waiting for Segment...", Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER);
    }

    private function drawTopBar(dc as Dc, segment as Segment) as Void {
        dc.setColor(Graphics.COLOR_DK_GRAY, Graphics.COLOR_TRANSPARENT);
        dc.fillRectangle(0, 0, dc.getWidth(), 30);
        
        dc.setColor(Graphics.COLOR_WHITE, Graphics.COLOR_TRANSPARENT);
        dc.drawText(dc.getWidth() / 2, 15, Graphics.FONT_XTINY, segment.name, Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER);
    }

    private function drawAheadBehind(dc as Dc, aheadBehind as Float) as Void {
        var color = (aheadBehind <= 0) ? Graphics.COLOR_GREEN : Graphics.COLOR_RED;
        var label = (aheadBehind <= 0) ? "AHEAD" : "BEHIND";
        var absTime = aheadBehind.abs();
        var minutes = (absTime / 60).toNumber();
        var seconds = (absTime.toNumber() % 60);
        var sign = (aheadBehind <= 0) ? "-" : "+";
        var timeStr = Lang.format("$1$$2$:$3$", [sign, minutes.format("%d"), seconds.format("%02d")]);

        dc.setColor(color, Graphics.COLOR_TRANSPARENT);
        dc.drawText(dc.getWidth() / 2, 70, Graphics.FONT_NUMBER_THAI_HOT, timeStr, Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER);
        
        dc.drawText(dc.getWidth() / 2, 115, Graphics.FONT_XTINY, label, Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER);
    }

    private function drawProgressBar(dc as Dc, segment as Segment, currentDist as Float) as Void {
        var y = 140;
        var width = dc.getWidth() - 40;
        var x = 20;
        var h = 6;

        // Background line
        dc.setColor(Graphics.COLOR_LT_GRAY, Graphics.COLOR_TRANSPARENT);
        dc.fillRectangle(x, y, width, h);

        var progress = (currentDist / segment.totalDistance);
        if (progress > 1.0) { progress = 1.0; }
        
        // Progress fill (YOU)
        dc.setColor(Graphics.COLOR_ORANGE, Graphics.COLOR_TRANSPARENT);
        dc.fillRectangle(x, y, (width * progress).toNumber(), h);

        // Marker for YOU (Triangle or Circle)
        dc.setColor(Graphics.COLOR_ORANGE, Graphics.COLOR_TRANSPARENT);
        dc.fillCircle(x + (width * progress).toNumber(), y + h/2, 6);

        // Marker for TARGET
        // Find where the target is at the current elapsed time
        var targetDist = 0.0f;
        var points = segment.points;
        for (var i = 1; i < points.size(); i++) {
            if (mElapsedSeconds <= points[i].elapsedSeconds) {
                var p1 = points[i-1];
                var p2 = points[i];
                var timeRatio = (mElapsedSeconds - p1.elapsedSeconds) / (p2.elapsedSeconds - p1.elapsedSeconds);
                targetDist = p1.distance + timeRatio * (p2.distance - p1.distance);
                break;
            }
            if (i == points.size() - 1) {
                targetDist = points[i].distance;
            }
        }
        var targetProgress = targetDist / segment.totalDistance;
        if (targetProgress > 1.0) { targetProgress = 1.0; }

        dc.setColor(Graphics.COLOR_WHITE, Graphics.COLOR_TRANSPARENT);
        dc.setPenWidth(2);
        dc.drawCircle(x + (width * targetProgress).toNumber(), y + h/2, 6);
        dc.setColor(Graphics.COLOR_BLACK, Graphics.COLOR_TRANSPARENT);
        dc.drawText(x + (width * targetProgress).toNumber(), y + h/2, Graphics.FONT_XTINY, "T", Graphics.TEXT_JUSTIFY_CENTER | Graphics.TEXT_JUSTIFY_VCENTER);
    }

    private function drawElevationProfile(dc as Dc, segment as Segment, currentDist as Float) as Void {
        var yBase = 190;
        var hMax = 30;
        var xStart = 20;
        var width = dc.getWidth() - 40;
        
        var points = segment.points;
        if (points.size() < 2) { return; }

        dc.setColor(Graphics.COLOR_DK_GRAY, Graphics.COLOR_TRANSPARENT);
        dc.setPenWidth(1);

        // Simple line-based elevation profile
        for (var i = 1; i < points.size(); i++) {
            var x1 = xStart + (points[i-1].distance / segment.totalDistance * width).toNumber();
            var x2 = xStart + (points[i].distance / segment.totalDistance * width).toNumber();
            
            // Normalize altitude (very roughly for this mock)
            var y1 = yBase - ((points[i-1].altitude - 100) / 50 * hMax).toNumber();
            var y2 = yBase - ((points[i].altitude - 100) / 50 * hMax).toNumber();
            
            dc.drawLine(x1, y1, x2, y2);
        }

        // Current position indicator in elevation
        var currentX = xStart + (currentDist / segment.totalDistance * width).toNumber();
        dc.setColor(Graphics.COLOR_ORANGE, Graphics.COLOR_TRANSPARENT);
        dc.drawLine(currentX, yBase - hMax, currentX, yBase);
    }

    private function drawBottomSection(dc as Dc, segment as Segment, currentDist as Float, currentSpeed as Float) as Void {
        var y = dc.getHeight() - 40;
        var midX = dc.getWidth() / 2;
        
        dc.setColor(Graphics.COLOR_LT_GRAY, Graphics.COLOR_TRANSPARENT);
        dc.drawLine(midX, y, midX, dc.getHeight());
        dc.drawLine(0, y, dc.getWidth(), y);

        var distToGo = segment.totalDistance - currentDist;
        if (distToGo < 0) { distToGo = 0.0f; }
        var distStr = (distToGo / 1000.0).format("%.2f") + "km";

        var timeToGo = 0;
        if (currentSpeed > 0.1) {
            timeToGo = (distToGo / currentSpeed).toNumber();
        }
        var tMinutes = timeToGo / 60;
        var tSeconds = timeToGo % 60;
        var timeStr = Lang.format("$1$:$2$", [tMinutes.format("%d"), tSeconds.format("%02d")]);

        var textColor = (getBackgroundColor() == Graphics.COLOR_BLACK) ? Graphics.COLOR_WHITE : Graphics.COLOR_BLACK;
        dc.setColor(textColor, Graphics.COLOR_TRANSPARENT);
        
        dc.drawText(midX / 2, y + 5, Graphics.FONT_XTINY, "DIST TO GO", Graphics.TEXT_JUSTIFY_CENTER);
        dc.drawText(midX / 2, y + 20, Graphics.FONT_TINY, distStr, Graphics.TEXT_JUSTIFY_CENTER);

        dc.drawText(midX + midX / 2, y + 5, Graphics.FONT_XTINY, "TIME TO GO", Graphics.TEXT_JUSTIFY_CENTER);
        dc.drawText(midX + midX / 2, y + 20, Graphics.FONT_TINY, timeStr, Graphics.TEXT_JUSTIFY_CENTER);
    }

    private function drawStatusIcons(dc as Dc) as Void {
        var padding = 5;
        var iconSize = 12;
        var width = dc.getWidth();
        var height = dc.getHeight();
        
        var app = Application.getApp() as LiveSegmentApp;
        
        // Sync status icon in bottom-left
        drawSyncIcon(dc, padding, height - padding - iconSize, iconSize, app.getSyncStatus());

        // Bluetooth icon in bottom-right
        drawBluetoothIcon(dc, width - padding - iconSize, height - padding - iconSize, iconSize, app.isPhoneConnected());
    }

    private function drawBluetoothIcon(dc as Dc, x as Number, y as Number, size as Number, connected as Boolean) as Void {
        if (connected) {
            dc.setColor(Graphics.COLOR_BLUE, Graphics.COLOR_TRANSPARENT);
        } else {
            dc.setColor(Graphics.COLOR_LT_GRAY, Graphics.COLOR_TRANSPARENT);
        }
        var half = size / 2;
        var quarter = size / 4;
        dc.setPenWidth(1);
        dc.drawLine(x + quarter, y, x + quarter, y + size);
        dc.drawLine(x + quarter, y, x + size - quarter, y + quarter);
        dc.drawLine(x + size - quarter, y + quarter, x + quarter, y + half);
        dc.drawLine(x + quarter, y + half, x + size - quarter, y + size - quarter);
        dc.drawLine(x + size - quarter, y + size - quarter, x + quarter, y + size);
    }

    private function drawSyncIcon(dc as Dc, x as Number, y as Number, size as Number, status as Symbol) as Void {
        var color = Graphics.COLOR_LT_GRAY;
        if (status == :syncing) { color = Graphics.COLOR_YELLOW; }
        else if (status == :success) { color = Graphics.COLOR_GREEN; }
        else if (status == :error) { color = Graphics.COLOR_RED; }
        
        dc.setColor(color, Graphics.COLOR_TRANSPARENT);
        var r = size / 4;
        var bottomY = y + size - r;
        dc.fillCircle(x + r, bottomY, r);
        dc.fillCircle(x + size - r, bottomY, r);
        dc.fillCircle(x + size/2, y + r + 1, r + 1);
        dc.fillRectangle(x + r, bottomY - r + 1, size - 2*r, r);
    }
}
