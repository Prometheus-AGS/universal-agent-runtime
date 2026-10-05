## ADDED Requirements
### Requirement: Canonical host extension envelope
UAR host selection readers SHALL consume draft.2 canonical extension {required,value}, where required is the outer boolean and value contains version/tools/servers and optional private workspacePath. Envelope and payload SHALL reject unknown fields. This repair SHALL NOT loosen the common extension schema or accept the old flat host shape. Existing tools/servers empty defaults, optional workspacePath, validity checks, saved binding/revision/workspace/server comparison and host tool admission SHALL remain unchanged.

#### Scenario: Canonical member extension
- **WHEN** a member declares {required:true,value:{version:1,tools:[],servers:[filesystem]}}
- **THEN** projection and turn resolution decode the same selection and preserve existing host-required and tool-policy semantics

#### Scenario: Canonical private binding extension
- **WHEN** trusted host attachment reads {required:true,value:{version:1,tools:[],servers:[filesystem],workspacePath:trustedPath}}
- **THEN** attachment compares the decoded private workspace and saved revision through existing admission checks

#### Scenario: Optional or malformed host requirement
- **WHEN** required is false, the envelope/payload is malformed, or required metadata is absent
- **THEN** the existing validity/diagnostic path remains authoritative and no host tool authority is granted
