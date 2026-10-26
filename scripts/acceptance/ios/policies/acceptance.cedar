// The simulator host admits only runtime-owned graph spawn/wait operations.
permit (
    principal == Agent::"orchestrator-agent",
    action == Action::"execute_tool",
    resource in [Tool::"spawn_agent", Tool::"wait_agents"]
);
