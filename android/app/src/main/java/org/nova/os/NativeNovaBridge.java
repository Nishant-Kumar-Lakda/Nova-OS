package org.nova.os;

import org.json.JSONObject;

/**
 * Optional JNI bridge to the Rust NEXUS and AIR implementations.
 *
 * Safe fallback: when the native library is not packaged yet, callers can
 * continue using the Android prototype router. The bridge itself never
 * executes Android operations; it only returns structured NOVA data.
 */
public final class NativeNovaBridge {
    private static final boolean AVAILABLE;

    static {
        boolean loaded;
        try {
            System.loadLibrary("nova_android_bridge");
            loaded = true;
        } catch (UnsatisfiedLinkError error) {
            loaded = false;
        }
        AVAILABLE = loaded;
    }

    private NativeNovaBridge() {
    }

    public static boolean isAvailable() {
        return AVAILABLE;
    }

    public static Result understand(String input) {
        if (!AVAILABLE) {
            return Result.unavailable();
        }

        try {
            String json = nativeUnderstand(input == null ? "" : input);
            if (json == null) {
                return Result.error("native bridge returned no result");
            }

            JSONObject object = new JSONObject(json);
            if (!object.optBoolean("ok", true)) {
                return Result.error(object.optString("error", "native understanding failed"));
            }

            return Result.success(
                    object.optString("action", ""),
                    (float) object.optDouble("confidence", 0.0),
                    json
            );
        } catch (Exception error) {
            return Result.error(error.getClass().getSimpleName() + ": " + error.getMessage());
        }
    }

    public static long recommendModelBudget(AndroidResourceSnapshot resources) {
        if (!AVAILABLE || resources == null) {
            return 0L;
        }
        try {
            return nativeRecommendModelBudget(
                    resources.availableMemoryBytes,
                    resources.batteryPercent,
                    resources.isLowMemory(),
                    resources.isLowBattery()
            );
        } catch (UnsatisfiedLinkError error) {
            return 0L;
        }
    }

    private static native String nativeUnderstand(String input);

    private static native long nativeRecommendModelBudget(
            long availableMemoryBytes,
            int batteryPercent,
            boolean lowMemory,
            boolean lowPower
    );

    public static final class Result {
        public final boolean available;
        public final boolean success;
        public final String action;
        public final float confidence;
        public final String rawJson;
        public final String error;

        private Result(
                boolean available,
                boolean success,
                String action,
                float confidence,
                String rawJson,
                String error
        ) {
            this.available = available;
            this.success = success;
            this.action = action;
            this.confidence = confidence;
            this.rawJson = rawJson;
            this.error = error;
        }

        static Result unavailable() {
            return new Result(false, false, "", 0.0f, "", "native library unavailable");
        }

        static Result success(String action, float confidence, String rawJson) {
            return new Result(true, true, action, confidence, rawJson, "");
        }

        static Result error(String error) {
            return new Result(true, false, "", 0.0f, "", error);
        }
    }
}
