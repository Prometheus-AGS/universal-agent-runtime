## MODIFIED Requirements

### Requirement: Provider settings remain usable at narrow widths
The Provider Overrides editor SHALL choose its provider-field composition from the available width of the provider panel rather than from the browser viewport width. It SHALL stack provider fields in one column when that panel cannot support the desktop composition and SHALL render exactly two columns when the panel can support it. Controls and long provider content MUST remain within the available page width without horizontal page scrolling or clipped keyboard focus.

#### Scenario: Provider settings are viewed in a narrow viewport
- **WHEN** the available provider-panel inline size is below the 36rem desktop-composition boundary, including when the browser viewport itself remains wide
- **THEN** provider fields stack into one column without horizontal page scrolling
- **AND** each visible provider control remains fully reachable and operable by keyboard

#### Scenario: Provider settings are viewed at desktop width
- **WHEN** the available provider-panel inline size is at or above the 36rem desktop-composition boundary
- **THEN** provider fields render in exactly two columns
- **AND** long provider content causes neither horizontal page scrolling nor clipped keyboard focus

#### Scenario: Available provider-panel width crosses the layout boundary
- **WHEN** the available provider-panel inline size changes across the 36rem desktop-composition boundary without unloading Provider Overrides
- **THEN** the provider fields switch between the one-column and two-column compositions according to the new panel width
- **AND** the current provider values and unsaved draft state remain unchanged
