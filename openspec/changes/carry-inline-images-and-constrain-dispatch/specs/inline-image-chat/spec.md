## Purpose
Preserve bounded admitted user images through ordinary chat execution without replacing them with text or exposing bytes in diagnostics. Transport evidence alone does not prove model understanding.

## ADDED Requirements

### Requirement: Typed ordinary image admission
The two ordinary OpenAI chat aliases SHALL admit ordered current-user text and inline PNG/JPEG image parts, including image-only input, through their supported actual compatible provider adapter. Existing text strings and text attachments SHALL preserve their meaning.

#### Scenario: Ordered current user parts
- **WHEN** a supported request contains text, PNG, text and JPEG in that order
- **THEN** the provider receives those parts with identical decoded image identities and order in streaming and non-streaming modes
- **AND** an image-only current turn remains present

#### Scenario: JSON-looking text stays text
- **WHEN** a caller submits a string containing JSON-looking image data
- **THEN** it remains ordinary text rather than being parsed as multimodal content

### Requirement: Bounded explicit image refusal
Image admission SHALL reject unsupported forms and malformed input before provider dispatch. The supported new form SHALL use standard padded-base64 PNG/JPEG, at most8images per current turn, the existing10MiB request envelope and100MiB decoder allocation ceiling. Non-document profiles SHALL refuse images.

#### Scenario: Invalid or excluded image input
- **WHEN** input has an unsupported URL/form/MIME/detail/part, invalid encoding, failed image decoding, excess count/size or an image attachment
- **THEN** it returns a bounded safe error and causes zero provider sends

#### Scenario: Ambiguous or ignored image replay
- **WHEN** convenience text or earlier client history would cause supplied image parts to be discarded
- **THEN** admission explicitly refuses rather than reporting text-only success

### Requirement: Canonical history and safe text projection
Admitted image identity/order SHALL survive supported warm history and governed checkpoint restoration. Matching, guardrails, memory and public run context SHALL receive complete ordered text without image bytes. Diagnostics SHALL not expose raw inline images or grant a generic secret-projection bypass.

#### Scenario: Retained image requires capable continuation
- **WHEN** a later text-only turn sends retained image history or a supported authorized checkpoint restores it
- **THEN** the final provider history retains the original image identities/order and requires image capability
- **AND** existing checkpoint admission/owner/claim controls still apply

### Requirement: Evidence-scoped vision claims
Public support SHALL distinguish implemented transport, deterministic wire evidence, genuine-model understanding and unqualified/excluded paths. Genuine vision qualification SHALL use a fresh private visual fact whose answer is not leaked through prompt text. Existing implemented paths SHALL not be removed solely for lack of qualification.

#### Scenario: Deterministic peer succeeds
- **WHEN** a deterministic peer confirms exact image bytes and returns canned text
- **THEN** the receipt credits transport only and does not certify visual understanding, a platform or a release
