package org.nova.os;

import org.json.JSONObject;

/**
 * Optional JNI bridge to the Rust NEXUS, AIR, and NOVA Core implementations.
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
            if (!object.optBoolean("ok", false)) {
                return Result.error(object.optString("error", "native understanding failed"));
            }

            JSONObject intent = object.optJSONObject("intent");
            if (intent == null) {
                return Result.error("native bridge returned no intent");
            }

            String action = intent.optString("action", "");
            float confidence = (float) intent.optDouble("confidence", 0.0);
            String parameter = "";
            JSONObject parameters = intent.optJSONObject("parameters");
            if (parameters != null) {
                parameter = parameters.optString("app", "");
            }

            return Result.success(action, confidence, parameter, json);
        } catch (Exception error) {
            return Result.error(error.getClass().getSimpleName() + ": " + error.getMessage());
        }
    }

    public static long recommendModelBudget(AndroidResourceSnapshot resources) {
        if (!AVAILABLE || resources == null || resources.batteryPercent < 0) {
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

    public static CoreStatus bootDiagnostics() {
        if (!AVAILABLE) {
            return CoreStatus.unavailable();
        }
        try {
            String json = nativeBootDiagnostics();
            if (json == null) {
                return CoreStatus.error("native boot diagnostics returned no result");
            }
            JSONObject object = new JSONObject(json);
            if (!object.optBoolean("ok", false)) {
                return CoreStatus.error(object.optString("error", "core boot failed"));
            }
            return CoreStatus.ready();
        } catch (Exception error) {
            return CoreStatus.error(error.getClass().getSimpleName() + ": " + error.getMessage());
        }
    }

    private static native String nativeUnderstand(String input);

    private static native long nativeRecommendModelBudget(
            long availableMemoryBytes,
            int batteryPercent,
            boolean lowMemory,
            boolean lowPower
    );

    private static native String nativeBootDiagnostics();

    public static final class Result {
        public final boolean available;
        public final boolean success;
        public final String action;
        public final float confidence;
        public final String parameter;
        public final String rawJson;
        public final String error;

        private Result(
                boolean available,
                boolean success,
                String action,
                float confidence,
                String parameter,
                String rawJson,
                String error
        ) {
            this.available = available;
            this.success = success;
            this.action = action;
            this.confidence = confidence;
            this.parameter = parameter;
            this.rawJson = rawJson;
            this.error = error;
        }

        static Result unavailable() {
            return new Result(false, false, "", 0.0f, "", "", "native library unavailable");
        }

        static Result success(String action, float confidence, String parameter, String rawJson) {
            return new Result(true, true, action, confidence, parameter, rawJson, "");
        }

        static Result error(String error) {
            return new Result(true, false, "", 0.0f, "", "", error);
        }
    }

    public static final class CoreStatus {
        public final boolean available;
        public final boolean ready;
        public final String error;

        private CoreStatus(boolean available, boolean ready, String error) {
            this.available = available;
            this.ready = ready;
            this.error = error;
        }

        static CoreStatus unavailable() {
            return new CoreStatus(false, false, "native library unavailable");
        }

        static CoreStatus ready() {
            return new CoreStatus(true, true, "");
        }

        static CoreStatus error(String error) {
            return new CoreStatus(true, false, error);
        }
    }
}
