package org.nova.os;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.os.BatteryManager;
import android.provider.Settings;

import java.util.Locale;

/**
 * First Android platform adapter.
 *
 * Only explicitly allowed, low-risk operations are implemented. Device state
 * changes such as Wi-Fi, Bluetooth, and flashlight control remain simulation-only.
 */
public final class AndroidPlatformAdapter implements NovaPlatformAdapter {
    private final Activity activity;

    public AndroidPlatformAdapter(Activity activity) {
        this.activity = activity;
    }

    @Override
    public String execute(NovaCommandRouter.Command command) throws Exception {
        switch (command.action) {
            case BATTERY_STATUS:
                BatteryManager battery = (BatteryManager) activity.getSystemService(Context.BATTERY_SERVICE);
                int percent = battery != null
                        ? battery.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
                        : -1;
                return "Battery: " + percent + "%";

            case OPEN_SETTINGS:
                activity.startActivity(new Intent(Settings.ACTION_SETTINGS));
                return "Android Settings opened.";

            case OPEN_CAMERA:
                activity.startActivity(new Intent("android.media.action.IMAGE_CAPTURE"));
                return "Camera launch requested.";

            case APP_OPEN:
                return openSafeAlias(command.parameter);

            case FLASHLIGHT_SIMULATE:
            case WIFI_SIMULATE:
            case BLUETOOTH_SIMULATE:
                return "SIMULATION ONLY. No device state was changed.";

            case UNKNOWN:
            default:
                throw new IllegalArgumentException("Unsupported Android action: " + command.action);
        }
    }

    private String openSafeAlias(String alias) {
        String normalized = alias == null ? "" : alias.trim().toLowerCase(Locale.ROOT);
        switch (normalized) {
            case "settings":
                activity.startActivity(new Intent(Settings.ACTION_SETTINGS));
                return "Android Settings opened.";
            case "camera":
                activity.startActivity(new Intent("android.media.action.IMAGE_CAPTURE"));
                return "Camera launch requested.";
            case "calculator":
                Intent calculator = activity.getPackageManager()
                        .getLaunchIntentForPackage("com.google.android.calculator");
                if (calculator == null) {
                    throw new IllegalArgumentException("Calculator application is not available.");
                }
                activity.startActivity(calculator);
                return "Calculator opened.";
            case "browser":
                Intent browser = new Intent(Intent.ACTION_VIEW);
                browser.setData(android.net.Uri.parse("https://example.invalid"));
                browser.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
                activity.startActivity(browser);
                return "Browser launch requested.";
            default:
                throw new IllegalArgumentException("App alias is not allowlisted: " + alias);
        }
    }
}
