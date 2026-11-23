## ADDED Requirements

### Requirement: Computation workers are side-effect free
A worker that computes UI data SHALL exchange only messages with its page and SHALL NOT
access the network, browser storage, the DOM, the entity graph store or any transport. It
SHALL import only pure modules.

#### Scenario: A computation worker runs
- **WHEN** a computation worker handles a request
- **THEN** it issues no network request and writes no browser storage

### Requirement: Catalog projection and filtering stay within a compute budget
The Models page SHALL record the duration of catalog projection and of catalog filtering
as User Timing measures named `uar:catalog-project` and `uar:catalog-filter`, and each
SHALL complete within `CATALOG_COMPUTE_BUDGET_MS` on the full provider catalog. Plan
fixes the constant from the measurement.

#### Scenario: Typing a query over the full catalog
- **WHEN** an operator types an eight-character query into the Models search box over the full catalog
- **THEN** every `uar:catalog-filter` measure is at most `CATALOG_COMPUTE_BUDGET_MS`
- **AND** no long task of 50 ms or more is attributed to the projection or filter step

### Requirement: Catalog filtering runs in a worker when the catalog worker is enabled
Where the catalog worker is enabled, the Models page SHALL run catalog filtering in a
dedicated module worker, post the catalog to it once per catalog change and per keystroke
post only the query and filters, receive ordered row keys, and discard any result whose
request identifier is not the latest.

#### Scenario: Fast typing shows only the latest result
- **WHEN** the operator types a query faster than the worker answers
- **THEN** only the result for the last request is displayed
- **AND** it equals the main-thread result for that query

### Requirement: Catalog worker results match the main-thread functions and fall back safely
Where the catalog worker is enabled, it SHALL call the same pure `projectModelRows` and
`filterModelRows` functions the main thread uses. If the worker cannot be created or
reports an error, the page SHALL compute the result on the main thread with identical
output and no user-visible error.

#### Scenario: The worker is unavailable
- **WHEN** worker construction fails in the browser
- **THEN** the Models page lists and filters the catalog with the same rows and order as with the worker

#### Scenario: Worker and main thread agree
- **WHEN** the same query and filters run with and without the worker
- **THEN** the ordered key lists are equal
