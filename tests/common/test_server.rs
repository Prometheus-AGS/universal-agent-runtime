use std::net::TcpListener;
use universal_agent_runtime::config::AppConfig;

/// Find an available port for test server.
pub fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Build a minimal `AppConfig` suitable for integration tests.
///
/// Uses in-memory SurrealDB (`mem://test`) and a random free port.
/// No MCP, no JWT, rate-limiting disabled, timeouts disabled.
pub fn test_config(port: u16) -> AppConfig {
    let yaml = format!(
        r#"
server:
  port: {port}
  host: "127.0.0.1"

security:
  jwt_required: false
  jwt_secret: "test_secret_key_must_be_at_least_32_chars_long"

resilience:
  rate_limit_enabled: false
  timeout_disabled: true
  requests_per_second: 1000.0
  burst_size: 1000.0

persistence:
  provider: "surreal"
  database_url: "mem://test"
  vector_dimension: 384
  external_cache_enabled: false

llm:
  model: "openai/gpt-4o"
  api_key: "test-key"

file_processing:
  provider: "auto"
  upload_dir: "/tmp/uar-test-uploads"
  max_files_per_prompt: 5
  max_file_size: 10485760
  max_total_size: 20971520

native_tools:
  file_tools_enabled: false
  file_allowed_paths: []
  web_fetch_enabled: false
  terminal_exec_enabled: false
  session_search_enabled: false

skill_evolution:
  enabled: false

acp:
  enabled: false
  path: "/acp"
  auth_required: false
  session_ttl_secs: 3600
"#
    );

    let cfg = config::Config::builder()
        .add_source(config::File::from_str(&yaml, config::FileFormat::Yaml))
        .build()
        .expect("test config should parse");

    cfg.try_deserialize::<AppConfig>()
        .expect("test config should deserialize into AppConfig")
}
