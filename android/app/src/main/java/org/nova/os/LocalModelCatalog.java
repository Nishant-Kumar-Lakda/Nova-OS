package org.nova.os;

import android.content.Context;

import java.io.File;

/**
 * App-private local model inventory. It never needs storage permissions and
 * never downloads model files. AIR remains responsible for model residency.
 */
public final class LocalModelCatalog {
    private final File modelDirectory;

    public LocalModelCatalog(Context context) {
        modelDirectory = new File(context.getFilesDir(), "models");
        if (!modelDirectory.exists()) {
            //noinspection ResultOfMethodCallIgnored
            modelDirectory.mkdirs();
        }
    }

    public String status() {
        File[] files = modelDirectory.listFiles();
        int count = files == null ? 0 : files.length;
        long bytes = 0L;
        if (files != null) {
            for (File file : files) {
                if (file.isFile()) {
                    bytes += file.length();
                }
            }
        }
        return "Local models: " + count + " file(s), " + bytes + " bytes";
    }

    public File directory() {
        return modelDirectory;
    }
}
