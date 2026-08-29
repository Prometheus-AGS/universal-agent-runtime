# Ingress for https://uar.know-me.tools
#
# Prerequisites (must already be installed on the cluster):
#   1. nginx ingress controller   — provides the "nginx" IngressClass
#   2. cert-manager               — handles ACME certificate issuance
#   3. A ClusterIssuer named "letsencrypt-prod" configured for ACME HTTP-01
#
# cert-manager will automatically provision and renew a TLS certificate
# from Let's Encrypt and store it in the "uar-tls" Secret.
# The nginx annotations enforce HTTPS redirect for all HTTP traffic.

resource "kubernetes_ingress_v1" "uar_ingress" {
  metadata {
    name      = "uar-ingress"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "uar"
      "app.kubernetes.io/component" = "ingress"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }

    annotations = {
      # ── Ingress class ───────────────────────────────────────────────────
      "kubernetes.io/ingress.class" = "nginx"

      # ── cert-manager — automatic TLS via Let's Encrypt ──────────────────
      "cert-manager.io/cluster-issuer" = "letsencrypt-prod"

      # ── SSL redirect ────────────────────────────────────────────────────
      # Redirect all HTTP → HTTPS (308 Permanent Redirect)
      "nginx.ingress.kubernetes.io/ssl-redirect"       = "true"
      "nginx.ingress.kubernetes.io/force-ssl-redirect" = "true"

      # ── Proxy settings ──────────────────────────────────────────────────
      # Increase timeouts for long-running LLM streaming responses.
      "nginx.ingress.kubernetes.io/proxy-read-timeout"    = "600"
      "nginx.ingress.kubernetes.io/proxy-send-timeout"    = "600"
      "nginx.ingress.kubernetes.io/proxy-connect-timeout" = "60"

      # Support Server-Sent Events (SSE) used by the UAR streaming API.
      "nginx.ingress.kubernetes.io/proxy-buffering" = "off"

      # Increase body size limit for file upload endpoints (50 MB default in UAR).
      "nginx.ingress.kubernetes.io/proxy-body-size" = "100m"
    }
  }

  spec {
    # ── TLS ──────────────────────────────────────────────────────────────
    tls {
      hosts       = ["uar.know-me.tools"]
      secret_name = "uar-tls"
    }

    # ── Routing rules ─────────────────────────────────────────────────────
    rule {
      host = "uar.know-me.tools"

      http {
        path {
          path      = "/"
          path_type = "Prefix"

          backend {
            service {
              name = kubernetes_service.uar_svc.metadata[0].name
              port {
                number = 1906
              }
            }
          }
        }
      }
    }
  }

  depends_on = [
    kubernetes_deployment.uar,
    kubernetes_service.uar_svc,
  ]
}

# ── Surrealist Web UI — surrealist.know-me.tools ─────────────────────────────
resource "kubernetes_ingress_v1" "surrealist_ingress" {
  metadata {
    name      = "surrealist-ingress"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "surrealist"
      "app.kubernetes.io/component" = "ingress"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }

    annotations = {
      "kubernetes.io/ingress.class"                    = "nginx"
      "cert-manager.io/cluster-issuer"                 = "letsencrypt-prod"
      "nginx.ingress.kubernetes.io/ssl-redirect"       = "true"
      "nginx.ingress.kubernetes.io/force-ssl-redirect" = "true"
      "nginx.ingress.kubernetes.io/proxy-read-timeout" = "60"
      "nginx.ingress.kubernetes.io/proxy-send-timeout" = "60"
    }
  }

  spec {
    tls {
      hosts       = ["surrealist.know-me.tools"]
      secret_name = "surrealist-tls"
    }

    rule {
      host = "surrealist.know-me.tools"

      http {
        path {
          path      = "/"
          path_type = "Prefix"

          backend {
            service {
              name = kubernetes_service.surrealist_svc.metadata[0].name
              port {
                number = 80
              }
            }
          }
        }
      }
    }
  }

  depends_on = [
    kubernetes_deployment.surrealist,
    kubernetes_service.surrealist_svc,
  ]
}

# ── SurrealDB WebSocket API — surreal-api.know-me.tools ─────────────────────
# Surrealist (running in the browser) connects DIRECTLY to SurrealDB over
# WebSocket. This ingress exposes the SurrealDB API endpoint externally so
# users can configure Surrealist to connect to wss://surreal-api.know-me.tools
resource "kubernetes_ingress_v1" "surreal_api_ingress" {
  metadata {
    name      = "surreal-api-ingress"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "surreal"
      "app.kubernetes.io/component" = "ingress"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }

    annotations = {
      "kubernetes.io/ingress.class"                    = "nginx"
      "cert-manager.io/cluster-issuer"                 = "letsencrypt-prod"
      "nginx.ingress.kubernetes.io/ssl-redirect"       = "true"
      "nginx.ingress.kubernetes.io/force-ssl-redirect" = "true"
      # Extended timeouts for long-lived WebSocket connections
      "nginx.ingress.kubernetes.io/proxy-read-timeout" = "3600"
      "nginx.ingress.kubernetes.io/proxy-send-timeout" = "3600"
      # Disable response buffering for WebSocket / streaming
      "nginx.ingress.kubernetes.io/proxy-buffering" = "off"
      # WebSocket upgrade — nginx ingress handles Upgrade/Connection headers
      # automatically when proxy-http-version is set to 1.1; no snippet needed.
      "nginx.ingress.kubernetes.io/proxy-http-version" = "1.1"
    }
  }

  spec {
    tls {
      hosts       = ["surreal-api.know-me.tools"]
      secret_name = "surreal-api-tls"
    }

    rule {
      host = "surreal-api.know-me.tools"

      http {
        path {
          path      = "/"
          path_type = "Prefix"

          backend {
            service {
              name = kubernetes_service.surreal_svc.metadata[0].name
              port {
                number = 8000
              }
            }
          }
        }
      }
    }
  }

  depends_on = [
    kubernetes_stateful_set.surreal,
    kubernetes_service.surreal_svc,
  ]
}

# ── DbGate PostgreSQL Admin — pg.know-me.tools ──────────────────────────────
resource "kubernetes_ingress_v1" "dbgate_ingress" {
  metadata {
    name      = "dbgate-ingress"
    namespace = kubernetes_namespace.uar.metadata[0].name

    labels = {
      "app.kubernetes.io/name"      = "dbgate"
      "app.kubernetes.io/component" = "ingress"
      "app.kubernetes.io/part-of"   = "universal-agent-runtime"
    }

    annotations = {
      "kubernetes.io/ingress.class"                    = "nginx"
      "cert-manager.io/cluster-issuer"                 = "letsencrypt-prod"
      "nginx.ingress.kubernetes.io/ssl-redirect"       = "true"
      "nginx.ingress.kubernetes.io/force-ssl-redirect" = "true"
      "nginx.ingress.kubernetes.io/proxy-read-timeout" = "300"
      "nginx.ingress.kubernetes.io/proxy-send-timeout" = "300"
      "nginx.ingress.kubernetes.io/proxy-body-size"    = "50m"
      # WebSocket upgrade — nginx ingress handles Upgrade/Connection headers
      # automatically when proxy-http-version is set to 1.1; no snippet needed.
      "nginx.ingress.kubernetes.io/proxy-http-version" = "1.1"
    }
  }

  spec {
    tls {
      hosts       = ["pg.know-me.tools"]
      secret_name = "dbgate-tls"
    }

    rule {
      host = "pg.know-me.tools"

      http {
        path {
          path      = "/"
          path_type = "Prefix"

          backend {
            service {
              name = kubernetes_service.dbgate_svc.metadata[0].name
              port {
                number = 3000
              }
            }
          }
        }
      }
    }
  }

  depends_on = [
    kubernetes_deployment.dbgate,
    kubernetes_service.dbgate_svc,
  ]
}
