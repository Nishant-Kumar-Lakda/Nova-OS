package org.nova.os;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Android-first NOVA orchestration facade.
 *
 * It mirrors the Rust architecture while JNI integration is still being
 * developed: NEXUS/router -> task session -> safety gate -> platform adapter.
 */
public final class NovaAndroidEngine {
    public static final class ExecutionResult {
        public final NovaTaskSession task;
        public final NovaSafetyGate.Decision decision;
        public final String message;

        ExecutionResult(NovaTaskSession task, NovaSafetyGate.Decision decision, String message) {
            this.task = task;
            this.decision = decision;
            this.message = message;
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
            return new ExecutionResult(null, NovaSafetyGate.Decision.REJECT,
                    "NOVA rejected the request: " + error.getMessage());
        }
        history.add(task);

        NovaCommandRouter.Command command = NovaCommandRouter.route(input);
        if (command.action == NovaCommandRouter.Action.UNKNOWN) {
            task.failed("NOVA does not understand this command yet.");
            return new ExecutionResult(task, NovaSafetyGate.Decision.REJECT,
                    "NOVA does not understand this command yet.");
        }

        task.understood(command);
        NovaSafetyGate.Decision decision = safetyGate.decide(command);
        if (decision == NovaSafetyGate.Decision.REJECT) {
            task.failed("Action rejected by safety policy.");
            return new ExecutionResult(task, decision, "Action rejected by NOVA safety policy.");
        }

        if (decision == NovaSafetyGate.Decision.SIMULATE) {
            task.ready();
            task.running();
            try {
                String message = platform.execute(command);
                task.completed(message);
                return new ExecutionResult(task, decision, message);
            } catch (Exception error) {
                task.failed(error.getMessage() == null ? error.getClass().getSimpleName() : error.getMessage());
                return new ExecutionResult(task, decision, "Simulation failed safely: " + error.getMessage());
            }
        }

        try {
            task.ready();
            task.running();
            String message = platform.execute(command);
            task.completed(message);
            return new ExecutionResult(task, decision, message);
        } catch (Exception error) {
            String message = error.getMessage() == null ? error.getClass().getSimpleName() : error.getMessage();
            task.failed(message);
            return new ExecutionResult(task, decision, "Execution blocked safely: " + message);
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
