package org.nova.os;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Android-first NOVA orchestration facade.
 *
 * Preferred path: Rust NEXUS through the optional JNI bridge. Safe fallback:
 * deterministic Java routing while the native library is not packaged.
 */
public final class NovaAndroidEngine {
    public static final class ExecutionResult {
        public final NovaTaskSession task;
        public final NovaSafetyGate.Decision decision;
        public final String message;
        public final boolean nativeNexusUsed;

        ExecutionResult(
                NovaTaskSession task,
                NovaSafetyGate.Decision decision,
                String message,
                boolean nativeNexusUsed
        ) {
            this.task = task;
            this.decision = decision;
            this.message = message;
            this.nativeNexusUsed = nativeNexusUsed;
        }
    }

    private final NovaPlatformAdapter platform;
    private final NovaSafetyGate safetyGate;
    private final List<NovaTaskSession> history = new ArrayList<>();
    private long nextTaskId = 1L;

    public NovaAndroidEngine(NovaPlatformAdapter platform) {
        this.platform = platform;
        this.safetyGate = new NovaSafetyGate();
    }

    public ExecutionResult execute(String input) {
        NovaTaskSession task;
        try {
            task = new NovaTaskSession(nextTaskId++, input);
        } catch (IllegalArgumentException error) {
            return new ExecutionResult(
                    null,
                    NovaSafetyGate.Decision.REJECT,
                    "NOVA rejected the request: " + error.getMessage(),
                    false
            );
        }
        history.add(task);

        NativeNovaBridge.Result nativeResult = NativeNovaBridge.understand(input);
        NovaCommandRouter.Command command;
        boolean nativeUsed = false;

        if (nativeResult.success) {
            command = NovaCommandRouter.fromNative(
                    input,
                    nativeResult.action,
                    nativeResult.confidence,
                    nativeResult.parameter
            );
            nativeUsed = true;
        } else {
            command = NovaCommandRouter.route(input);
        }

        if (command.action == NovaCommandRouter.Action.UNKNOWN) {
            task.failed("NOVA does not understand this command yet.");
            String source = nativeResult.available ? "Rust NEXUS" : "safe fallback router";
            return new ExecutionResult(
                    task,
                    NovaSafetyGate.Decision.REJECT,
                    "NOVA does not understand this command yet (" + source + ").",
                    nativeUsed
            );
        }

        task.understood(command);
        NovaSafetyGate.Decision decision = safetyGate.decide(command);
        if (decision == NovaSafetyGate.Decision.REJECT) {
            task.failed("Action rejected by safety policy.");
            return new ExecutionResult(
                    task,
                    decision,
                    "Action rejected by NOVA safety policy.",
                    nativeUsed
            );
        }

        if (decision == NovaSafetyGate.Decision.SIMULATE) {
            task.ready();
            task.running();
            try {
                String message = platform.execute(command);
                task.completed(message);
                return new ExecutionResult(task, decision, message, nativeUsed);
            } catch (Exception error) {
                String message = error.getMessage() == null
                        ? error.getClass().getSimpleName()
                        : error.getMessage();
                task.failed(message);
                return new ExecutionResult(
                        task,
                        decision,
                        "Simulation failed safely: " + message,
                        nativeUsed
                );
            }
        }

        try {
            task.ready();
            task.running();
            String message = platform.execute(command);
            task.completed(message);
            return new ExecutionResult(task, decision, message, nativeUsed);
        } catch (Exception error) {
            String message = error.getMessage() == null
                    ? error.getClass().getSimpleName()
                    : error.getMessage();
            task.failed(message);
            return new ExecutionResult(
                    task,
                    decision,
                    "Execution blocked safely: " + message,
                    nativeUsed
            );
        }
    }

    public List<NovaTaskSession> history() {
        return Collections.unmodifiableList(history);
    }

    public void cancelLatest() {
        if (!history.isEmpty()) {
            history.get(history.size() - 1).cancel();
        }
    }
}
