package org.nova.os;

import java.util.Locale;

/**
 * Deterministic fallback router for Android safe mode.
 *
 * The preferred path is the Rust NEXUS JNI bridge. This router remains as a
 * fallback so the APK still works before the native library is packaged.
 */
public final class NovaCommandRouter {
    public enum Action {
        BATTERY_STATUS,
        OPEN_SETTINGS,
        OPEN_CAMERA,
        APP_OPEN,
        FLASHLIGHT_SIMULATE,
        WIFI_SIMULATE,
        BLUETOOTH_SIMULATE,
        UNKNOWN
    }

    public static final class Command {
        public final Action action;
        public final float confidence;
        public final String originalText;
        public final String parameter;

        public Command(Action action, float confidence, String originalText) {
            this(action, confidence, originalText, "");
        }

        public Command(Action action, float confidence, String originalText, String parameter) {
            this.action = action;
            this.confidence = confidence;
            this.originalText = originalText;
            this.parameter = parameter == null ? "" : parameter;
        }
    }

    private NovaCommandRouter() {
    }

    public static Command fromNative(String input, String action, float confidence) {
        Action mapped;
        switch (action) {
            case "battery.status":
                mapped = Action.BATTERY_STATUS;
                break;
            case "settings.open":
                mapped = Action.OPEN_SETTINGS;
                break;
            case "camera.open":
                mapped = Action.OPEN_CAMERA;
                break;
            case "app.open":
                mapped = Action.APP_OPEN;
                break;
            case "flashlight.on":
            case "flashlight.off":
                mapped = Action.FLASHLIGHT_SIMULATE;
                break;
            case "wifi.enable":
            case "wifi.disable":
                mapped = Action.WIFI_SIMULATE;
                break;
            case "bluetooth.enable":
            case "bluetooth.disable":
                mapped = Action.BLUETOOTH_SIMULATE;
                break;
            default:
                mapped = Action.UNKNOWN;
                break;
        }
        return new Command(mapped, confidence, input);
    }

    public static Command route(String input) {
        if (input == null || input.trim().isEmpty()) {
            return new Command(Action.UNKNOWN, 0.0f, "");
        }

        String text = input.trim().toLowerCase(Locale.ROOT);

        if (text.equals("battery") || text.contains("battery status") || text.contains("check battery")) {
            return new Command(Action.BATTERY_STATUS, 0.99f, input);
        }
        if (text.equals("settings") || text.equals("open settings")) {
            return new Command(Action.OPEN_SETTINGS, 0.99f, input);
        }
        if (text.equals("camera") || text.equals("open camera")) {
            return new Command(Action.OPEN_CAMERA, 0.99f, input);
        }
        if (text.contains("flashlight") || text.contains("torch")) {
            return new Command(Action.FLASHLIGHT_SIMULATE, 0.99f, input);
        }
        if (text.contains("wifi") || text.contains("wi-fi")) {
            return new Command(Action.WIFI_SIMULATE, 0.99f, input);
        }
        if (text.contains("bluetooth")) {
            return new Command(Action.BLUETOOTH_SIMULATE, 0.99f, input);
        }

        return new Command(Action.UNKNOWN, 0.30f, input);
    }
}
