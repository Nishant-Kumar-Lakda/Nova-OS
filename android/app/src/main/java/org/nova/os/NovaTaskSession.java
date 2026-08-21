package org.nova.os;

import java.util.Objects;

/** Android-side task session used until the Rust core is bridged through JNI. */
public final class NovaTaskSession {
    public enum State {
        CREATED,
        UNDERSTOOD,
        READY,
        RUNNING,
        COMPLETED,
        FAILED,
        CANCELLED
    }

    private final long id;
    private final String input;
    private final long createdAtMillis;
    private State state;
    private NovaCommandRouter.Command command;
    private String resultMessage;
    private String errorMessage;

    public NovaTaskSession(long id, String input) {
        if (input == null || input.trim().isEmpty()) {
            throw new IllegalArgumentException("Task input cannot be empty");
        }
        this.id = id;
        this.input = input.trim();
        this.createdAtMillis = System.currentTimeMillis();
        this.state = State.CREATED;
    }

    public long getId() {
        return id;
    }

    public String getInput() {
        return input;
    }

    public long getCreatedAtMillis() {
        return createdAtMillis;
    }

    public State getState() {
        return state;
    }

    public NovaCommandRouter.Command getCommand() {
        return command;
    }

    public String getResultMessage() {
        return resultMessage;
    }

    public String getErrorMessage() {
        return errorMessage;
    }

    void understood(NovaCommandRouter.Command command) {
        this.command = Objects.requireNonNull(command, "command");
        this.state = State.UNDERSTOOD;
    }

    void ready() {
        requireState(State.UNDERSTOOD);
        this.state = State.READY;
    }

    void running() {
        requireState(State.READY);
        this.state = State.RUNNING;
    }

    void completed(String message) {
        requireState(State.RUNNING);
        this.resultMessage = message;
        this.state = State.COMPLETED;
    }

    void failed(String message) {
        this.errorMessage = message;
        this.state = State.FAILED;
    }

    public void cancel() {
        if (state == State.COMPLETED || state == State.FAILED || state == State.CANCELLED) {
            return;
        }
        this.state = State.CANCELLED;
    }

    private void requireState(State expected) {
        if (state != expected) {
            throw new IllegalStateException("Invalid task state: " + state + " -> expected " + expected);
        }
    }
}
