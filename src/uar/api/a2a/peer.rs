//! Immutable host configuration for authenticated UAR peers.

use std::collections::BTreeMap;

use secrecy::ExposeSecret;

use super::client::A2AClient;

#[derive(Clone)]
pub struct TrustedA2APeer {
    pub instance_id: String,
    pub agent_id: String,
    pub endpoint: String,
    pub client: A2AClient,
}

impl std::fmt::Debug for TrustedA2APeer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrustedA2APeer")
            .field("instance_id", &self.instance_id)
            .field("agent_id", &self.agent_id)
            .field("endpoint", &self.endpoint)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Default)]
pub struct TrustedA2APeers {
    source_instance_id: String,
    peers: BTreeMap<String, TrustedA2APeer>,
}

impl std::fmt::Debug for TrustedA2APeers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrustedA2APeers")
            .field("source_instance_id", &self.source_instance_id)
            .field("peer_count", &self.peers.len())
            .finish()
    }
}

impl TrustedA2APeers {
    pub(crate) fn from_config(config: &crate::config::A2aConfig) -> Self {
        let peers = config
            .trusted_peers
            .iter()
            .map(|peer| {
                let binding = TrustedA2APeer {
                    instance_id: peer.instance_id.clone(),
                    agent_id: peer.agent_id.clone(),
                    endpoint: peer.endpoint.clone(),
                    client: A2AClient::new().with_bearer_token(peer.bearer_token.expose_secret()),
                };
                (peer.endpoint.clone(), binding)
            })
            .collect();
        Self {
            source_instance_id: config.instance_id.clone(),
            peers,
        }
    }

    pub fn source_instance_id(&self) -> &str {
        &self.source_instance_id
    }

    pub fn resolve(&self, endpoint: &str, agent_id: &str) -> anyhow::Result<TrustedA2APeer> {
        let peer = self
            .peers
            .get(endpoint)
            .ok_or_else(|| anyhow::anyhow!("A2A endpoint is not a configured trusted UAR peer"))?;
        anyhow::ensure!(
            peer.agent_id == agent_id,
            "A2A peer endpoint is bound to another agent"
        );
        Ok(peer.clone())
    }
}
