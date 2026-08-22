package org.nova.os;

import android.app.Activity;
import android.os.Bundle;
import android.view.Gravity;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

import java.io.File;
import java.io.IOException;
import java.util.Locale;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/** Android-first NOVA prototype shell. */
public class MainActivity extends Activity {
    private TextView output;
    private TextView status;
    private NovaAndroidEngine engine;
    private LocalModelCatalog modelCatalog;
    private ExecutorService background;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        background = Executors.newSingleThreadExecutor();
        modelCatalog = new LocalModelCatalog(this);
        engine = new NovaAndroidEngine(new AndroidPlatformAdapter(this));
        buildUi();
        refreshStatus();
    }

    @Override
    protected void onDestroy() {
        if (background != null) {
            background.shutdownNow();
        }
        super.onDestroy();
    }

    private void buildUi() {
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(32, 28, 32, 28);

        TextView title = new TextView(this);
        title.setText("NOVA OS");
        title.setTextSize(30);
        title.setGravity(Gravity.CENTER_HORIZONTAL);

        TextView mode = new TextView(this);
        mode.setText("ANDROID FIRST • SAFE MODE • OFFLINE");
        mode.setTextSize(15);
        mode.setGravity(Gravity.CENTER_HORIZONTAL);
        mode.setPadding(0, 8, 0, 18);

        status = new TextView(this);
        status.setTextSize(13);
        status.setGravity(Gravity.CENTER_HORIZONTAL);
        status.setPadding(0, 0, 0, 18);

        EditText command = new EditText(this);
        command.setHint("Try: battery status, open settings, ask NOVA anything");
        command.setSingleLine(true);

        Button execute = new Button(this);
        execute.setText("Run NOVA");
        execute.setOnClickListener(v -> handle(command.getText().toString()));

        Button cancel = new Button(this);
        cancel.setText("Cancel Latest Task");
        cancel.setOnClickListener(v -> {
            engine.cancelLatest();
            output.setText("Latest NOVA task cancelled.");
        });

        Button diagnostics = new Button(this);
        diagnostics.setText("Run Core Diagnostics");
        diagnostics.setOnClickListener(v -> runDiagnosticsAsync());

        Button models = new Button(this);
        models.setText("Initialize Local AI Model");
        models.setOnClickListener(v -> initializeLocalModelAsync());

        output = new TextView(this);
        output.setTextSize(16);
        output.setPadding(0, 28, 0, 28);
        output.setText("NOVA ready. No command is executed automatically.");

        TextView examples = new TextView(this);
        examples.setText("Safe commands:\n"
                + "• battery status\n"
                + "• open settings\n"
                + "• open camera\n"
                + "• open calculator\n"
                + "• ask NOVA: explain what airplane mode does\n"
                + "• turn on flashlight (simulation)\n"
                + "• turn on Wi-Fi (simulation)\n\n"
                + "No Internet, accessibility, device-admin, SMS, contacts, microphone, Wi-Fi control, Bluetooth control, or flashlight-control permissions are requested.");
        examples.setTextSize(14);

        root.addView(title, matchWrap());
        root.addView(mode, matchWrap());
        root.addView(status, matchWrap());
        root.addView(command, matchWrap());
        root.addView(execute, matchWrap());
        root.addView(cancel, matchWrap());
        root.addView(diagnostics, matchWrap());
        root.addView(models, matchWrap());
        root.addView(output, matchWrap());
        root.addView(examples, matchWrap());

        ScrollView scroll = new ScrollView(this);
        scroll.addView(root);
        setContentView(scroll);
    }

    private void initializeLocalModelAsync() {
        output.setText("Preparing local AI model…");
        background.execute(() -> {
            try {
                File model = NativeModelBridge.ensureBundledModel(this);
                runOnUiThread(() -> output.setText(
                        "Local AI model ready.\n\n"
                                + model.getName() + "\n"
                                + model.length() + " bytes\n\n"
                                + "No network connection is used at runtime."
                ));
            } catch (IOException error) {
                runOnUiThread(() -> output.setText(
                        "Unable to initialize local model: " + error.getMessage()
                ));
            }
        });
    }

    private void runDiagnosticsAsync() {
        output.setText("Running NOVA Core diagnostics…");
        background.execute(() -> {
            String result = coreDiagnostics();
            runOnUiThread(() -> output.setText(result));
        });
    }

    private void refreshStatus() {
        AndroidResourceSnapshot resources = AndroidResourceSnapshot.read(this);
        long budget = NativeNovaBridge.recommendModelBudget(resources);
        NativeNovaBridge.CoreStatus core = NativeNovaBridge.bootDiagnostics();
        String source = NativeNovaBridge.isAvailable() ? "Rust/JNI" : "Java fallback";
        String coreStatus = core.ready ? "READY" : (core.available ? "ERROR" : "FALLBACK");
        String modelStatus = NativeModelBridge.isAvailable() ? "READY" : "UNAVAILABLE";
        String budgetText = budget > 0
                ? String.format(Locale.ROOT, "%d MB", budget / (1024 * 1024))
                : "pending";

        status.setText(String.format(
                Locale.ROOT,
                "NEXUS: %s • Core: %s • AI: %s • RAM: %d MB • Battery: %d%% • AIR budget: %s",
                source,
                coreStatus,
                modelStatus,
                resources.availableMemoryBytes / (1024 * 1024),
                resources.batteryPercent,
                budgetText
        ));
    }

    private String coreDiagnostics() {
        refreshStatus();
        NativeNovaBridge.CoreStatus core = NativeNovaBridge.bootDiagnostics();
        if (core.ready) {
            return "NOVA Rust Core booted successfully.\n\n"
                    + "Core: READY\n"
                    + "AIR: READY\n"
                    + "Planner: READY\n"
                    + "Memory: READY\n"
                    + "Context: READY\n"
                    + "Runtime: READY\n"
                    + "Built-in Skills: READY\n"
                    + "Local AI bridge: " + (NativeModelBridge.isAvailable() ? "READY" : "UNAVAILABLE") + "\n\n"
                    + modelCatalog.status();
        }
        return "Native NOVA Core is not ready.\n\nReason: " + core.error;
    }

    private void handle(String input) {
        refreshStatus();
        output.setText("Processing…");
        background.execute(() -> {
            NovaAndroidEngine.ExecutionResult result = engine.execute(input);
            String rendered = renderResult(input, result);
            runOnUiThread(() -> output.setText(rendered));
        });
    }

    private String renderResult(String input, NovaAndroidEngine.ExecutionResult result) {
        if (result.task == null) {
            return result.message;
        }

        NovaTaskSession task = result.task;
        NovaCommandRouter.Command command = task.getCommand();
        StringBuilder text = new StringBuilder();
        text.append("NOVA TASK #").append(task.getId()).append("\n\n");
        text.append("Input: ").append(task.getInput()).append("\n");
        if (command != null) {
            text.append("NIL action: ")
                    .append(command.action.name().toLowerCase(Locale.ROOT))
                    .append("\n");
            text.append(String.format(Locale.ROOT, "Confidence: %.2f\n", command.confidence));
            if (!command.parameter.isEmpty()) {
                text.append("Parameter: ").append(command.parameter).append("\n");
            }
        }
        text.append("NEXUS source: ")
                .append(result.nativeNexusUsed ? "Rust/JNI" : "Android fallback")
                .append("\n");
        text.append("Decision: ").append(result.decision).append("\n");
        text.append("State: ").append(task.getState()).append("\n\n");
        text.append(result.message);

        if (result.decision == NovaSafetyGate.Decision.REJECT && NativeModelBridge.isAvailable()
                && input != null && !input.trim().isEmpty()) {
            NativeModelBridge.Result ai = NativeModelBridge.generate(this, input, 64);
            if (ai.success) {
                text.append("\n\nOffline AI response:\n").append(ai.text.trim());
            } else {
                text.append("\n\nOffline AI unavailable: ").append(ai.error);
            }
        }

        return text.toString();
    }

    private LinearLayout.LayoutParams matchWrap() {
        return new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
        );
    }
}
