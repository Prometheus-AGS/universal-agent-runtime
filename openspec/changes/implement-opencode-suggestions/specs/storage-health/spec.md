# Spec Delta: Storage Health

## MODIFIED Requirements

### Requirement: Storage Monitoring

The system SHALL monitor PGlite storage usage and provide visual feedback to the user.

#### Scenario: High Usage Warning

- **Given** PGlite storage usage exceeds 90% of the quota
- **When** the `storage-health` component renders
- **Then** it MUST display a warning state (e.g., red color, "Storage Low" label).

#### Scenario: UI Integration

- **Given** the application is loaded
- **When** the user views the sidebar
- **Then** the `storage-health` indicator MUST be visible in the footer.
