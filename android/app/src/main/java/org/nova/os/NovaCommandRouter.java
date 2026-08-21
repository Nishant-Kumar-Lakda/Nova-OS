package org.nova.os;

import java.util.Locale;

/**
 * Deterministic command router for the first Android prototype.
 * This layer deliberately uses no network, model, or privileged API.
 */
public final class NovaCommandRouter {
    public enum Action {
        BATTERY_STATUS,
        OPEN_SETTINGS,
        OPEN_CAMERA,
        FLASHLIGHT_SIMULATE,
        WIFI_SIMULATE,
        BLUETOOTH_SIMULATE,
        UNKNOWN
    }

    public static final class Command {
        public final Action action;
        public final float confidence;
        public final String originalText;

        public Command(Action action, float confidence, String originalText) {
            this.action = action;
            this.confidence = confidence;
            this.originalText = originalText;
        }
    }

    private NovaCommandRouter() {
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
