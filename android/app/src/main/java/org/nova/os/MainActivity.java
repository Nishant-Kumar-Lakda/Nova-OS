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

import java.util.Locale;

/** Android-first NOVA prototype shell. */
public class MainActivity extends Activity {
    private TextView output;
    private NovaAndroidEngine engine;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        engine = new NovaAndroidEngine(new AndroidPlatformAdapter(this));
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
        mode.setText("ANDROID FIRST • SAFE MODE • OFFLINE");
        mode.setTextSize(15);
        mode.setGravity(Gravity.CENTER_HORIZONTAL);
        mode.setPadding(0, 8, 0, 24);

        EditText command = new EditText(this);
        command.setHint("Try: battery status, open settings, open camera");
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

        output = new TextView(this);
        output.setTextSize(16);
        output.setPadding(0, 28, 0, 28);
        output.setText("NOVA ready. No command is executed automatically.");

        TextView examples = new TextView(this);
        examples.setText("Safe commands:\n"
                + "• battery status\n"
                + "• open settings\n"
                + "• open camera\n"
                + "• turn on flashlight (simulation)\n"
                + "• turn on Wi-Fi (simulation)\n\n"
                + "This prototype requests no Internet, accessibility, device-admin, SMS, contacts, microphone, Wi-Fi control, Bluetooth control, or flashlight-control permissions.");
        examples.setTextSize(14);

        root.addView(title, matchWrap());
        root.addView(mode, matchWrap());
        root.addView(command, matchWrap());
        root.addView(execute, matchWrap());
        root.addView(cancel, matchWrap());
        root.addView(output, matchWrap());
        root.addView(examples, matchWrap());

        ScrollView scroll = new ScrollView(this);
        scroll.addView(root);
        setContentView(scroll);
    }

    private void handle(String input) {
        NovaAndroidEngine.ExecutionResult result = engine.execute(input);
        if (result.task == null) {
            output.setText(result.message);
            return;
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
        }
        text.append("Decision: ").append(result.decision).append("\n");
        text.append("State: ").append(task.getState()).append("\n\n");
        text.append(result.message);
        output.setText(text.toString());
    }

    private LinearLayout.LayoutParams matchWrap() {
        return new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
        );
    }
}
