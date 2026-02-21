resource "kubernetes_namespace" "uar" {
  metadata {
    name = "uar"

    labels = {
      "app.kubernetes.io/managed-by" = "opentofu"
      "app.kubernetes.io/part-of"    = "universal-agent-runtime"
    }
  }
}
