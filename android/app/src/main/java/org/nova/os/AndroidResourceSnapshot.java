package org.nova.os;

import android.app.ActivityManager;
import android.content.Context;
import android.os.BatteryManager;

/**
 * Read-only Android resource information used by NOVA's resource policy.
 * No special permissions are required for these values.
 */
public final class AndroidResourceSnapshot {
    public final long availableMemoryBytes;
    public final long totalMemoryBytes;
    public final int batteryPercent;

    private AndroidResourceSnapshot(long availableMemoryBytes, long totalMemoryBytes, int batteryPercent) {
        this.availableMemoryBytes = availableMemoryBytes;
        this.totalMemoryBytes = totalMemoryBytes;
        this.batteryPercent = batteryPercent;
    }

    public static AndroidResourceSnapshot read(Context context) {
        ActivityManager manager = (ActivityManager) context.getSystemService(Context.ACTIVITY_SERVICE);
        ActivityManager.MemoryInfo memoryInfo = new ActivityManager.MemoryInfo();
        if (manager != null) {
            manager.getMemoryInfo(memoryInfo);
        }

        BatteryManager battery = (BatteryManager) context.getSystemService(Context.BATTERY_SERVICE);
        int batteryPercent = battery != null
                ? battery.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
                : -1;

        return new AndroidResourceSnapshot(
                memoryInfo.availMem,
                memoryInfo.totalMem,
                batteryPercent
        );
    }

    public boolean isLowMemory() {
        return availableMemoryBytes > 0 && availableMemoryBytes < 256L * 1024L * 1024L;
    }

    public boolean isLowBattery() {
        return batteryPercent >= 0 && batteryPercent < 20;
    }
}
