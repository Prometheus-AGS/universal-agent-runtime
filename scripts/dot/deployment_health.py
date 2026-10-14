"""Narrow read-only post-deployment HTTP health, separate from product suites."""
from acceptance import Host, require


def run_deployment_health(spec, evidence):
    require(evidence.artifact['kind'] == 'independently_installed_publication',
            'deployment health must name an actual independently installed publication')
    observations = []
    for label, host_spec in spec['hosts'].items():
        # These endpoints identify already deployed, operator-selected runtimes.
        # This stage never starts, stops or replaces their processes.
        host = Host(host_spec, evidence, label)
        health, _ = host.request('GET', '/health', principal=None)
        card = host.api('GET', '/.well-known/agent-card.json', principal=None)
        require(card['version'] == evidence.artifact['tag'].removeprefix('v'), 'deployed card release differs')
        host.api('GET', '/api/uar/runs', principal=None, expect=401)
        observations.append({'host': label, 'base_url': host.base, 'health': health,
                             'card_version': card['version'], 'private_anonymous_status': 401})
    evidence.observe('deployment-http-health', {'hosts': observations,
        'claim_scope': 'selected deployed HTTP endpoints reachable with matching card version and private auth boundary',
        'rollout_and_running_image_identity': 'not_observed_by_http_health'})
