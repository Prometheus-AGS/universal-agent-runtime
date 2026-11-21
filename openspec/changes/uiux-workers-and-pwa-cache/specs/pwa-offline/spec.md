## ADDED Requirements

### Requirement: Hashed build assets are retained for the current and previous build only
The service worker SHALL keep cached content-hashed `/assets/` files for at most two
builds: the build named by the HTML it most recently received from the network and the
build before it. It SHALL delete files that only older builds reference when it first
observes a newer build. Build identity SHALL be derived from the hashed entry script that
the received HTML references.

#### Scenario: Assets of older builds are evicted
- **WHEN** builds A, B and C are loaded in turn on one profile
- **THEN** after C has loaded, Cache Storage holds hashed asset entries for builds C and B
- **AND** it holds no entry that only build A referenced

#### Scenario: The current build loads offline after eviction
- **WHEN** the network is disabled after build C has loaded and the app is reloaded
- **THEN** the shell and build C's hashed assets are served from Cache Storage and the app renders

#### Scenario: The previous build's chunk is still served
- **WHEN** build C has been observed and the network is disabled
- **THEN** a request for a cached hashed chunk of build B succeeds from the cache
- **AND** a request for a hashed file that only build A referenced fails

#### Scenario: The entry script cannot be identified
- **WHEN** the received HTML references no hashed entry script
- **THEN** the service worker caches no `/assets/` file and serves every request from the network

### Requirement: Superseded service worker caches are deleted on upgrade
When a new service worker activates, it SHALL delete every Cache Storage cache whose
name it does not own, including the legacy cache named `uar-v1`.

#### Scenario: A profile with the legacy cache upgrades
- **WHEN** a profile that holds the legacy `uar-v1` cache receives the new service worker and it activates
- **THEN** `caches.keys()` lists no `uar-v1`
- **AND** the app loads when the network is available

### Requirement: Unhashed static files are revalidated
The service worker SHALL serve unhashed same-origin images and fonts stale-while-revalidate
so a file changed by a deploy is served no later than the second load after the change.
It SHALL NOT serve any unhashed script or stylesheet from the cache before the network.

#### Scenario: A deployed unhashed image changes
- **WHEN** an unhashed image such as `/brand/uar-favicon-dark.svg` changes on the server after it was cached
- **THEN** the second load after the change serves the new bytes

#### Scenario: An unhashed script outside the assets directory changes
- **WHEN** a script served from a path outside `/assets/` changes on the server after a previous load
- **THEN** the next load serves the new bytes

### Requirement: Navigations share one cached shell
The service worker SHALL store at most one HTML document for navigations, under the
application shell key, and SHALL answer an offline navigation to any same-origin SPA route
with that shell.

#### Scenario: Many distinct URLs are visited
- **WHEN** the operator visits twenty distinct SPA URLs, some with query strings
- **THEN** Cache Storage holds one HTML entry in total

#### Scenario: A deep link is reloaded offline
- **WHEN** the network is disabled and the operator reloads a deep link such as `/admin/models`
- **THEN** the shell renders and the client router shows that route

### Requirement: API and stream requests bypass the service worker
The service worker SHALL NOT intercept or cache non-GET requests or requests whose path
begins with `/api/`, `/v1/` or `/metrics`.

#### Scenario: An event stream stays open
- **WHEN** the console holds an event stream under `/api/` open and the server emits events
- **THEN** the events reach the page as they are emitted
- **AND** Cache Storage contains no URL under `/api/`
