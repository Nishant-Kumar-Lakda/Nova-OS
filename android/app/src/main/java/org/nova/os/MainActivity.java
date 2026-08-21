package org.nova.os;

import android.app.Activity;
import android.content.Intent;
import android.os.BatteryManager;
import android.os.Bundle;
import android.provider.Settings;
import android.view.Gravity;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

import java.util.Locale;

/**
 * NOVA Android 0.1 safe-mode prototype.
 *
 * Safety rules:
 * - No INTERNET permission.
 * - No Accessibility service.
 * - No device-admin privileges.
 * - No background service.
 * - Wi-Fi/Bluetooth/flashlight commands are simulation-only.
 * - Only battery status and benign system/camera launches are executed.
 */
public class MainActivity extends Activity {
    private TextView output;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        buildUi();
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
        mode.setText("SAFE MODE • Offline • No privileged access");
        mode.setTextSize(15);
        mode.setGravity(Gravity.CENTER_HORIZONTAL);
        mode.setPadding(0, 8, 0, 24);

        EditText command = new EditText(this);
        command.setHint("Try: battery status, open settings, open camera");
        command.setSingleLine(true);

        Button execute = new Button(this);
        execute.setText("Run NOVA Command");
        execute.setOnClickListener(v -> handle(command.getText().toString()));

        output = new TextView(this);
        output.setTextSize(16);
        output.setPadding(0, 28, 0, 28);
        output.setText("Ready. Nothing runs until you press the button.");

        TextView examples = new TextView(this);
        examples.setText("Safe test commands:\n• battery status\n• open settings\n• open camera\n• turn on flashlight (simulated)\n• turn on Wi-Fi (simulated)");
        examples.setTextSize(14);

        root.addView(title, matchWrap());
        root.addView(mode, matchWrap());
        root.addView(command, matchWrap());
        root.addView(execute, matchWrap());
        root.addView(output, matchWrap());
        root.addView(examples, matchWrap());

        ScrollView scroll = new ScrollView(this);
        scroll.addView(root);
        setContentView(scroll);
    }

    private LinearLayout.LayoutParams matchWrap() {
        return new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
        );
    }

    private void handle(String input) {
        NovaCommandRouter.Command command = NovaCommandRouter.route(input);
        String action = command.action.name().toLowerCase(Locale.ROOT);

        StringBuilder plan = new StringBuilder();
        plan.append("NEXUS prototype result\n\n");
        plan.append("Input: ").append(input).append("\n");
        plan.append("Action: ").append(action).append("\n");
        plan.append(String.format(Locale.ROOT, "Confidence: %.2f\n\n", command.confidence));

        try {
            switch (command.action) {
                case BATTERY_STATUS:
                    BatteryManager battery = (BatteryManager) getSystemService(BATTERY_SERVICE);
                    int percent = battery != null
                            ? battery.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
                            : -1;
                    plan.append("Executed safely. Battery: ").append(percent).append("%.");
                    output.setText(plan.toString());
                    break;

                case OPEN_SETTINGS:
                    startActivity(new Intent(Settings.ACTION_SETTINGS));
                    plan.append("Executed safely: Android Settings opened.");
                    output.setText(plan.toString());
                    break;

                case OPEN_CAMERA:
                    startActivity(new Intent("android.media.action.IMAGE_CAPTURE"));
                    plan.append("Executed safely: camera launch requested.");
                    output.setText(plan.toString());
                    break;

                case FLASHLIGHT_SIMULATE:
                case WIFI_SIMULATE:
                case BLUETOOTH_SIMULATE:
                    plan.append("SIMULATION ONLY. No device state was changed.");
                    output.setText(plan.toString());
                    break;

                case UNKNOWN:
                default:
                    plan.append("No action executed. NOVA does not understand this command yet.");
                    output.setText(plan.toString());
                    break;
            }
        } catch (Exception error) {
            plan.append("\n\nExecution blocked safely: ").append(error.getClass().getSimpleName());
            output.setText(plan.toString());
        }
    }
}
