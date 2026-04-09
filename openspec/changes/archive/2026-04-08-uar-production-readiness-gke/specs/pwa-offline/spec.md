## ADDED Requirements

### Requirement: Service worker for offline asset caching
The frontend SHALL register a service worker that caches static assets for offline availability.

#### Scenario: Assets cached on first load
- **WHEN** the user loads the app for the first time
- **THEN** the service worker caches all JS, CSS, and HTML assets

#### Scenario: Offline access to cached pages
- **WHEN** the user loses network connectivity
- **THEN** previously visited pages load from the service worker cache

#### Scenario: Online-only features gracefully degrade
- **WHEN** the user is offline and attempts to send a chat message
- **THEN** the UI displays a "You are offline" indicator and queues the message for retry
