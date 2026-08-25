## ADDED Requirements

### Requirement: Provider settings controls expose complete accessible context
The Provider Overrides surface SHALL expose every provider card as a named group and SHALL give each visible field, enabled switch, and API-key reveal action a provider-specific programmatic name. Help text and invalid-state recovery text MUST be programmatically associated with the affected control.

#### Scenario: Provider controls are traversed with assistive technology
- **WHEN** an operator navigates provider settings without relying on visual layout
- **THEN** Base URL, Protocol, API Key, Default Model, Enabled, and API-key reveal controls are identifiable for the correct provider
- **AND** help or invalid-state text is included in the affected control's accessible description

#### Scenario: Provider default model is unavailable
- **WHEN** the stored default model is not currently selectable
- **THEN** the model control exposes an invalid state and the associated recovery guidance to assistive technology

### Requirement: Provider settings communicate asynchronous outcomes
The Provider Overrides surface SHALL expose loading and successful-save feedback as polite, atomic status updates and SHALL expose failures as alerts. A rejected save MUST NOT emit successful-save feedback and MUST preserve pending drafts.

#### Scenario: Provider settings load or save succeeds
- **WHEN** provider settings are loading or a save completes successfully
- **THEN** the corresponding visible message is available as a polite status update

#### Scenario: Provider settings operation fails
- **WHEN** a provider settings load or save operation fails
- **THEN** the visible error is announced as an alert
- **AND** a failed save retains the unsaved provider drafts

### Requirement: Provider settings protect unsaved drafts
The Provider Overrides surface SHALL derive its modified state from the authoritative provider settings draft. Save MUST be disabled while no provider draft exists; Refresh MUST be disabled while drafts exist or a provider settings operation is in flight; and browser unload MUST request confirmation while drafts exist.

#### Scenario: Provider settings are clean
- **WHEN** no provider settings draft exists and no operation is in flight
- **THEN** Save is disabled and Refresh is available

#### Scenario: Provider settings are modified
- **WHEN** one or more provider settings drafts exist
- **THEN** Save is enabled, Refresh is disabled, and visible text identifies each modified provider
- **AND** the operator is told to save changes before refreshing

#### Scenario: Browser unload is attempted with drafts
- **WHEN** browser navigation or window closure would unload provider settings while drafts exist
- **THEN** the browser unload event is cancelled so the browser can request confirmation

#### Scenario: Provider save succeeds
- **WHEN** all provider drafts are saved successfully
- **THEN** the dirty indicators clear, Save becomes disabled, and Refresh becomes available

#### Scenario: Provider save fails
- **WHEN** saving provider drafts fails
- **THEN** the dirty indicators remain and Refresh stays disabled

### Requirement: Provider settings remain usable at narrow widths
The Provider Overrides editor SHALL stack provider fields in one column at narrow widths and SHALL retain its two-column composition at desktop widths. Controls and long provider content MUST remain within the available viewport without clipping keyboard focus.

#### Scenario: Provider settings are viewed in a narrow viewport
- **WHEN** the available provider-panel width cannot support the desktop field composition
- **THEN** fields stack into one column without horizontal page scrolling
- **AND** controls remain fully keyboard accessible

#### Scenario: Provider settings are viewed at desktop width
- **WHEN** the available provider-panel width supports the incumbent desktop composition
- **THEN** provider fields render in two columns
