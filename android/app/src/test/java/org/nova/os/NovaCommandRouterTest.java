package org.nova.os;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class NovaCommandRouterTest {
    @Test
    public void routesBattery() {
        NovaCommandRouter.Command result = NovaCommandRouter.route("check battery");
        assertEquals(NovaCommandRouter.Action.BATTERY_STATUS, result.action);
        assertTrue(result.confidence >= 0.95f);
    }

    @Test
    public void routesSettingsCaseInsensitively() {
        NovaCommandRouter.Command result = NovaCommandRouter.route("Open Settings");
        assertEquals(NovaCommandRouter.Action.OPEN_SETTINGS, result.action);
    }

    @Test
    public void routesCamera() {
        NovaCommandRouter.Command result = NovaCommandRouter.route("open camera");
        assertEquals(NovaCommandRouter.Action.OPEN_CAMERA, result.action);
    }

    @Test
    public void routesDangerousCapabilitiesToSimulationOnlyActions() {
        assertEquals(
                NovaCommandRouter.Action.WIFI_SIMULATE,
                NovaCommandRouter.route("turn on Wi-Fi").action
        );
        assertEquals(
                NovaCommandRouter.Action.BLUETOOTH_SIMULATE,
                NovaCommandRouter.route("turn off Bluetooth").action
        );
        assertEquals(
                NovaCommandRouter.Action.FLASHLIGHT_SIMULATE,
                NovaCommandRouter.route("turn on flashlight").action
        );
    }

    @Test
    public void rejectsEmptyAndUnknownInput() {
        assertEquals(NovaCommandRouter.Action.UNKNOWN, NovaCommandRouter.route("").action);
        assertEquals(NovaCommandRouter.Action.UNKNOWN, NovaCommandRouter.route("do magic").action);
        assertTrue(NovaCommandRouter.route("do magic").confidence < 0.75f);
    }

    @Test
    public void neverThrowsForNullInput() {
        assertEquals(NovaCommandRouter.Action.UNKNOWN, NovaCommandRouter.route(null).action);
    }
}
