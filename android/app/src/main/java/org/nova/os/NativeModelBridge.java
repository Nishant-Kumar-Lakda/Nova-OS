package org.nova.os;

import android.content.Context;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * Local GGUF model bridge.
 *
 * The model is packaged into the APK at build time and copied into the app's
 * private directory. No network permission or runtime download is used.
 */
public final class NativeModelBridge {
    public static final String MODEL_ASSET = "models/SmolLM2-135M-Instruct-Q2_K.gguf";
    private static final String MODEL_FILE = "SmolLM2-135M-Instruct-Q2_K.gguf";
    private static final long MINIMUM_MODEL_BUDGET_BYTES = 256_000_000L;
    private static final boolean AVAILABLE;

    static {
        boolean loaded;
        try {
            System.loadLibrary("nova_llama");
            loaded = true;
        } catch (UnsatisfiedLinkError error) {
            loaded = false;
        }
        AVAILABLE = loaded;
    }

    private NativeModelBridge() {
    }

    public static boolean isAvailable() {
        return AVAILABLE;
    }

    public static File ensureBundledModel(Context context) throws IOException {
        File model = new File(context.getFilesDir(), "models/" + MODEL_FILE);
        if (model.isFile() && model.length() > 0) {
            return model;
        }

        File parent = model.getParentFile();
        if (parent != null && !parent.exists() && !parent.mkdirs()) {
            throw new IOException("Unable to create private model directory");
        }

        File temporary = new File(parent, MODEL_FILE + ".part");
        try {
            try (InputStream input = context.getAssets().open(MODEL_ASSET);
                 FileOutputStream output = new FileOutputStream(temporary)) {
                byte[] buffer = new byte[64 * 1024];
                int read;
                while ((read = input.read(buffer)) != -1) {
                    output.write(buffer, 0, read);
                }
                output.getFD().sync();
            }

            if (!temporary.isFile() || temporary.length() <= 0) {
                throw new IOException("Bundled model asset is empty");
            }

            if (!temporary.renameTo(model)) {
                if (model.exists() && model.delete() && temporary.renameTo(model)) {
                    return model;
                }
                throw new IOException("Unable to finalize bundled model file");
            }
            return model;
        } finally {
            if (temporary.exists()) {
                //noinspection ResultOfMethodCallIgnored
                temporary.delete();
            }
        }
    }

    public static Result generate(Context context, String prompt, int maxTokens) {
        if (!AVAILABLE) {
            return Result.error("native llama.cpp library unavailable");
        }
        if (prompt == null || prompt.trim().isEmpty()) {
            return Result.error("prompt cannot be empty");
        }

        try {
            AndroidResourceSnapshot resources = AndroidResourceSnapshot.read(context);
            long budget = NativeNovaBridge.recommendModelBudget(resources);
            if (budget > 0 && budget < MINIMUM_MODEL_BUDGET_BYTES) {
                return Result.error("AIR memory policy is below the minimum model budget");
            }

            File model = ensureBundledModel(context);
            int threads = Math.max(1, Math.min(Runtime.getRuntime().availableProcessors(), 4));
            String text = nativeGenerate(model.getAbsolutePath(), prompt, maxTokens, threads);
            if (text == null) {
                return Result.error("native inference returned no result");
            }
            if (text.startsWith("ERROR:")) {
                return Result.error(text.substring("ERROR:".length()).trim());
            }
            return Result.success(text);
        } catch (Exception error) {
            return Result.error(error.getClass().getSimpleName() + ": " + error.getMessage());
        }
    }

    private static native String nativeGenerate(
            String modelPath,
            String prompt,
            int maxTokens,
            int threads
    );

    public static final class Result {
        public final boolean success;
        public final String text;
        public final String error;

        private Result(boolean success, String text, String error) {
            this.success = success;
            this.text = text;
            this.error = error;
        }

        static Result success(String text) {
            return new Result(true, text, "");
        }

        static Result error(String error) {
            return new Result(false, "", error);
        }
    }
}
