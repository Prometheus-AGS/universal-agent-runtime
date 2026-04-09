## ADDED Requirements

### Requirement: Auto-generated OpenAPI documentation
The server SHALL expose `GET /api/docs` serving an interactive OpenAPI/Swagger UI and `GET /api/openapi.json` serving the OpenAPI 3.1 specification.

#### Scenario: Swagger UI accessible
- **WHEN** a browser navigates to `/api/docs`
- **THEN** an interactive Swagger UI is rendered showing all API endpoints

#### Scenario: OpenAPI spec downloadable
- **WHEN** a GET request is made to `/api/openapi.json`
- **THEN** the server returns a valid OpenAPI 3.1 JSON document describing all public endpoints

#### Scenario: Spec includes request/response schemas
- **WHEN** the OpenAPI spec is generated
- **THEN** each endpoint includes request body schemas, response schemas, and authentication requirements
