// Tool Approval Policy — require user approval for high-risk tool operations.
//
// Tools matching these patterns will pause execution and request
// explicit user approval before proceeding.
//
// The RunManager checks tool names against a heuristic list:
//   - delete, remove, write, drop, truncate, destroy
//
// To extend or override the heuristic with Cedar policy rules,
// add forbid rules targeting specific tools:

// Example: Require approval for filesystem write operations
// forbid (
//     principal,
//     action == Action::"execute_tool",
//     resource == Tool::"filesystem__write"
// ) unless {
//     context.approved == true
// };

// Example: Require approval for any tool that modifies external state
// forbid (
//     principal,
//     action == Action::"execute_tool",
//     resource
// ) when {
//     resource.requires_approval == true
// } unless {
//     context.approved == true
// };

// Default: permit all tool calls (the heuristic check in RunManager
// handles approval for dangerous tools without Cedar rules).
// Add specific forbid rules above to enforce Cedar-based approval.
permit (
    principal,
    action == Action::"execute_tool",
    resource
);
