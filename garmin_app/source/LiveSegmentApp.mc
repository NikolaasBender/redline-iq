import Toybox.Application;
import Toybox.Lang;
import Toybox.WatchUi;

class LiveSegmentApp extends Application.AppBase {

    private var mSyncStatus as Symbol = :none;
    private var mMockPhoneConnected as Boolean = System.getDeviceSettings().phoneConnected;
    private var mCloudSyncer as CloudSyncer = new CloudSyncer();
    private var mSegmentTracker as SegmentTracker = new SegmentTracker();
    private var mSegments as Array<Segment> = [] as Array<Segment>;

    function initialize() {
        AppBase.initialize();
    }

    function getSyncStatus() as Symbol {
        return mSyncStatus;
    }

    function setSyncStatus(status as Symbol) as Void {
        mSyncStatus = status;
        if (status == :success) {
            setupMockSegment(0.0); // Default to 0 for mock
        }
        WatchUi.requestUpdate();
    }

    function getSegmentTracker() as SegmentTracker {
        return mSegmentTracker;
    }

    function setSegments(segments as Array<Segment>) as Void {
        mSegments = segments;
        mSegmentTracker.setSegments(segments);
        WatchUi.requestUpdate();
    }

    private function setupMockSegment(startDist as Float) as Void {
        var points = [
            new SegmentPoint(0.0, 100.0, 0.0),
            new SegmentPoint(500.0, 110.0, 60.0),
            new SegmentPoint(1000.0, 125.0, 130.0),
            new SegmentPoint(1500.0, 120.0, 200.0),
            new SegmentPoint(2000.0, 140.0, 280.0)
        ];
        var segment = new Segment("Old La Honda", 2000.0, 280, points);
        setSegments([segment]);
        mSegmentTracker.setStartDistance(startDist);
    }

    function isPhoneConnected() as Boolean {
        return mMockPhoneConnected;
    }

    function toggleMockPhoneConnection() as Void {
        mMockPhoneConnected = !mMockPhoneConnected;
        WatchUi.requestUpdate();
    }

    function triggerSync() as Void {
        // Use a dummy token for now
        mCloudSyncer.fetchSegments("dummy_token", method(:onSyncComplete));
    }

    function onSyncComplete(responseCode as Number, data as Dictionary?) as Void {
        // Handle result (SyncStatus is already updated by CloudSyncer)
        System.println("Sync complete. Response: " + responseCode);
    }

    // onStart() is called on application start up
    function onStart(state as Dictionary?) as Void {
    }

    // onStop() is called when your application is exiting
    function onStop(state as Dictionary?) as Void {
    }

    // Return the initial view of your application here
    function getInitialView() as Array<Views or InputDelegates>? {
        return [ new LiveSegmentView(), new LiveSegmentDelegate() ] as Array<Views or InputDelegates>;
    }

}
