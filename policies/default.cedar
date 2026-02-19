// Default governance policy — permit all actions (permissive baseline)
//
// This policy allows any agent to perform any action on any resource.
// Add restrictive policies alongside this file to narrow permissions.

permit (
    principal,
    action,
    resource
);
