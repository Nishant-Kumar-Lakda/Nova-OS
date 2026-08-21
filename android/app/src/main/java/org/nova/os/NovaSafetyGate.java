package org.nova.os;

import java.util.Locale;

/**
 * Android prototype policy gate. It is intentionally stricter than the
 * eventual Rust policy until the complete native execution bridge is in place.
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
            case APP_OPEN:
                return isSafeAppAlias(command.parameter)
                        ? Decision.ALLOW
                        : Decision.REJECT;
            case FLASHLIGHT_SIMULATE:
            case WIFI_SIMULATE:
            case BLUETOOTH_SIMULATE:
                return Decision.SIMULATE;
            case UNKNOWN:
            default:
                return Decision.REJECT;
        }
    }

    private boolean isSafeAppAlias(String parameter) {
        if (parameter == null) {
            return false;
        }
        switch (parameter.trim().toLowerCase(Locale.ROOT)) {
            case "calculator":
            case "settings":
            case "camera":
                return true;
            default:
                return false;
        }
    }
}
