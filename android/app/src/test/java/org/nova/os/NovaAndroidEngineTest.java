package org.nova.os;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

public class NovaAndroidEngineTest {
    private static final class FakePlatform implements NovaPlatformAdapter {
        @Override
        public String execute(NovaCommandRouter.Command command) {
            return "FAKE:" + command.action.name();
        }
    }

    @Test
    public void batteryCommandCreatesCompletedTask() {
        NovaAndroidEngine engine = new NovaAndroidEngine(new FakePlatform());

        NovaAndroidEngine.ExecutionResult result = engine.execute("battery status");

        assertNotNull(result.task);
        assertEquals(NovaSafetyGate.Decision.ALLOW, result.decision);
        assertEquals(NovaTaskSession.State.COMPLETED, result.task.getState());
        assertEquals("FAKE:BATTERY_STATUS", result.message);
    }

    @Test
    public void flashlightRemainsSimulationOnly() {
        NovaAndroidEngine engine = new NovaAndroidEngine(new FakePlatform());

        NovaAndroidEngine.ExecutionResult result = engine.execute("turn on flashlight");

        assertEquals(NovaSafetyGate.Decision.SIMULATE, result.decision);
        assertEquals(NovaTaskSession.State.COMPLETED, result.task.getState());
        assertEquals("FAKE:FLASHLIGHT_SIMULATE", result.message);
    }

    @Test
    public void unknownCommandIsRejected() {
        NovaAndroidEngine engine = new NovaAndroidEngine(new FakePlatform());

        NovaAndroidEngine.ExecutionResult result = engine.execute("do magic");

        assertEquals(NovaSafetyGate.Decision.REJECT, result.decision);
        assertEquals(NovaTaskSession.State.FAILED, result.task.getState());
    }

    @Test
    public void emptyInputIsRejectedWithoutPlatformCall() {
        NovaAndroidEngine engine = new NovaAndroidEngine(new FakePlatform());

        NovaAndroidEngine.ExecutionResult result = engine.execute("   ");

        assertEquals(NovaSafetyGate.Decision.REJECT, result.decision);
        assertEquals(null, result.task);
    }

    @Test
    public void historyKeepsTaskOrder() {
        NovaAndroidEngine engine = new NovaAndroidEngine(new FakePlatform());
        engine.execute("battery status");
        engine.execute("open settings");

        assertEquals(2, engine.history().size());
        assertEquals(1L, engine.history().get(0).getId());
        assertEquals(2L, engine.history().get(1).getId());
    }
}
