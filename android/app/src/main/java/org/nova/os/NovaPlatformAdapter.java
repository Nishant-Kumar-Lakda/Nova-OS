package org.nova.os;

/** Platform boundary for Android capabilities. Core task logic never calls Android APIs directly. */
public interface NovaPlatformAdapter {
    String execute(NovaCommandRouter.Command command) throws Exception;
}
