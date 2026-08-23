## ADDED Requirements

### Requirement: Native supervisor cancellation joins process shutdown
An external native-service cancellation token SHALL initiate the same run cancellation, listener drain, resource cleanup, and completion path as an ordinary supported process stop signal without introducing a second cleanup implementation.

#### Scenario: Windows SCM stop is received
- **WHEN** the Windows service adapter cancels the server
- **THEN** UAR drains and releases resources through the existing graceful shutdown coordinator before the service reports Stopped
