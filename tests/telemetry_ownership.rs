//! Process-global ownership fixtures. Each case gets a fresh test process;
//! no fixture replaces a recorder/subscriber installed by another case.
use std::process::Command;
use universal_agent_runtime::{config::LogFormat, uar::telemetry};

#[test]
fn telemetry_ownership_isolated_processes() {
    #[cfg(feature = "telemetry")]
    let cases = [
        "fresh",
        "repeated",
        "foreign-tracing",
        "disabled-export",
        "foreign-metrics",
        "shutdown-restart",
        "exporter-offline",
    ];
    #[cfg(not(feature = "telemetry"))]
    let cases = ["fresh", "repeated", "foreign-tracing", "disabled-export"];
    for case in cases {
        let mut command = Command::new(std::env::current_exe().expect("test executable"));
        command
            .args([
                "--exact",
                "telemetry_ownership_child",
                "--ignored",
                "--nocapture",
            ])
            .env("UAR_TELEMETRY_FIXTURE", case)
            .env_remove("OTEL_EXPORTER_OTLP_ENDPOINT")
            .env_remove("UAR_LLM__TRACING")
            .env_remove("UAR_LOG_FILE")
            .env_remove("UAR_SIDECAR")
            .env_remove("RUST_LOG")
            .env_remove("UAR_TELEMETRY_EXPORTER_CHILD");
        if case == "disabled-export" {
            command
                .env(
                    "OTEL_EXPORTER_OTLP_ENDPOINT",
                    "http://127.0.0.1:9/v1/traces",
                )
                .env("UAR_LLM__TRACING", "false");
        }
        let output = command.output().expect("isolated telemetry fixture");
        assert!(
            output.status.success(),
            "case {case}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("TELEMETRY_FIXTURE_COMPLETE"),
            "fixture case {case} did not complete"
        );
    }
}

#[test]
#[ignore = "only the isolated-process parent selects this fixture"]
fn telemetry_ownership_child() {
    let case = std::env::var("UAR_TELEMETRY_FIXTURE").expect("fixture selector");
    assert_eq!(
        telemetry::status().subscriber,
        telemetry::SubscriberStatus::Uninitialized
    );
    match case.as_str() {
        "fresh" | "repeated" | "disabled-export" => {
            let provider = telemetry::init(&LogFormat::Compact, "info").expect("init");
            assert!(provider.is_none(), "offline default has no exporter");
            assert_eq!(
                telemetry::status().subscriber,
                telemetry::SubscriberStatus::Installed
            );
            assert_eq!(
                telemetry::status().export,
                telemetry::ExportStatus::Disabled
            );
            telemetry::metrics::init();
            #[cfg(feature = "telemetry")]
            {
                assert_eq!(
                    telemetry::metrics::status(),
                    telemetry::metrics::MetricsStatus::Installed
                );
                telemetry::metrics::record_request(
                    "GET",
                    "/fixture/{id}",
                    200,
                    std::time::Duration::from_millis(1),
                );
                let first = telemetry::metrics::scrape_handle().expect("owned handle");
                assert!(first.render().contains("uar_requests_total"));
                telemetry::metrics::init();
                assert!(std::ptr::eq(
                    first,
                    telemetry::metrics::scrape_handle().expect("same handle")
                ));
            }
            #[cfg(not(feature = "telemetry"))]
            assert_eq!(
                telemetry::metrics::status(),
                telemetry::metrics::MetricsStatus::Disabled
            );
            assert!(
                telemetry::init(&LogFormat::Json, "trace")
                    .expect("repeat")
                    .is_none()
            );
            assert_eq!(
                telemetry::status().subscriber,
                telemetry::SubscriberStatus::Installed
            );
            tracing::info!("offline repeated instance remains usable");
        }
        "foreign-tracing" => {
            tracing::subscriber::set_global_default(tracing_subscriber::registry())
                .expect("host subscriber");
            assert!(
                telemetry::init(&LogFormat::Pretty, "info")
                    .expect("coexist")
                    .is_none()
            );
            assert_eq!(
                telemetry::status().subscriber,
                telemetry::SubscriberStatus::HostOwned
            );
            #[cfg(feature = "telemetry")]
            assert_eq!(
                telemetry::status().export,
                telemetry::ExportStatus::HostOwned
            );
            assert!(
                telemetry::init(&LogFormat::Compact, "info")
                    .expect("repeat host")
                    .is_none()
            );
            assert!(tracing::dispatcher::get_default(|dispatch| {
                dispatch.is::<tracing_subscriber::Registry>()
            }));
        }
        #[cfg(feature = "telemetry")]
        "foreign-metrics" => {
            let host = metrics_exporter_prometheus::PrometheusBuilder::new()
                .install_recorder()
                .expect("host recorder");
            telemetry::metrics::init();
            assert_eq!(
                telemetry::metrics::status(),
                telemetry::metrics::MetricsStatus::HostOwned
            );
            assert!(telemetry::metrics::scrape_handle().is_none());
            telemetry::metrics::record_request(
                "GET",
                "/fixture/{id}",
                200,
                std::time::Duration::from_millis(1),
            );
            assert!(
                host.render().contains("uar_requests_total"),
                "host receives UAR writes"
            );
            telemetry::metrics::init();
            assert!(telemetry::metrics::scrape_handle().is_none());
        }
        #[cfg(feature = "telemetry")]
        "shutdown-restart" | "exporter-offline" => exporter_child(),
        _ => panic!("unknown telemetry fixture case"),
    }
    println!("TELEMETRY_FIXTURE_COMPLETE");
}

#[cfg(feature = "telemetry")]
fn exporter_child() {
    // Environment belongs to a new grandchild, so no unsafe process-global
    // environment mutation is needed even inside the Rust test runner.
    if std::env::var_os("UAR_TELEMETRY_EXPORTER_CHILD").is_none() {
        let output = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "telemetry_ownership_child",
                "--ignored",
                "--nocapture",
            ])
            .env("UAR_TELEMETRY_EXPORTER_CHILD", "1")
            .env(
                "OTEL_EXPORTER_OTLP_ENDPOINT",
                "http://127.0.0.1:9/v1/traces",
            )
            .output()
            .expect("offline exporter child");
        assert!(
            output.status.success(),
            "exporter child: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let runtime = tokio::runtime::Runtime::new().expect("exporter runtime");
    let provider = {
        let _entered = runtime.enter();
        telemetry::init(&LogFormat::Compact, "info").expect("exporter init does not fail execution")
    };
    tracing::info!("agent execution remains usable with unavailable collector");
    if let Some(provider) = provider {
        assert_eq!(telemetry::status().export, telemetry::ExportStatus::Enabled);
        let outcome = telemetry::shutdown(&provider);
        // Collector delivery is not guaranteed; failed flush must remain explicit.
        assert_eq!(
            telemetry::status().export,
            if outcome.is_ok() {
                telemetry::ExportStatus::Shutdown
            } else {
                telemetry::ExportStatus::Unavailable
            }
        );
        assert!(
            telemetry::init(&LogFormat::Compact, "info")
                .expect("restart instance")
                .is_none()
        );
        assert_eq!(
            telemetry::status().subscriber,
            telemetry::SubscriberStatus::Installed
        );
    } else {
        assert_eq!(
            telemetry::status().export,
            telemetry::ExportStatus::Unavailable
        );
    }
}
