## MODIFIED Requirements

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

#### Scenario: Non-http(s) request schemes are never cached
- **WHEN** the fetch handler processes a request whose URL scheme is not
  `http:` or `https:` (e.g. a `chrome-extension://` resource fetched by a
  browser extension's content script)
- **THEN** the service worker returns the fetched response to the caller
  without attempting `cache.put()`, and no error is thrown or logged
