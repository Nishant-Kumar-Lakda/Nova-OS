package org.nova.os;

/**
 * Android prototype policy gate. This is intentionally stricter than the
 * eventual Rust policy until the JNI bridge is in place.
 */
public final class NovaSafetyGate {
    public enum Decision {
        ALLOW,
        SIMULATE,
        REJECT
    }

    public Decision decide(NovaCommandRouter.Command command) {
        switch (command.action) {
            case BATTERY_STATUS:
            case OPEN_SETTINGS:
            case OPEN_CAMERA:
                return Decision.ALLOW;
            case FLASHLIGHT_SIMULATE:
            case WIFI_SIMULATE:
            case BLUETOOTH_SIMULATE:
                return Decision.SIMULATE;
            case UNKNOWN:
            default:
                return Decision.REJECT;
        }
    }
}
