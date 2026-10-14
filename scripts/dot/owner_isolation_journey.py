"""Real HTTP owner/thread/adjacent contracts; no fixture database writes.

The live host authenticates same-issuer tenant principals and rejects a different
issuer. Decoding JWT claims only checks fixture shape. Cross-issuer authenticated
storage isolation belongs to the separate-host real integration target.
"""
import base64
import copy
import json
from pathlib import Path
import uuid
from acceptance import poll, require
from journeys import decision, finish, pending, segment, tool_input


def claims(host, principal):
    encoded = host.credential(principal).split('.')[1]
    return json.loads(base64.urlsafe_b64decode(encoded + '=' * (-len(encoded) % 4)))


def run_owner_isolation(host, spec, evidence):
    foreign = ('other_tenant',)
    own = claims(host, 'owner')
    tenant, issuer = (claims(host, name) for name in ('other_tenant', 'other_issuer'))
    require(own['sub'] == tenant['sub'] == issuer['sub'], 'fixture requires identical subject')
    require(own.get('iss') == tenant.get('iss') and own.get('tenant_id') != tenant.get('tenant_id'), 'foreign tenant fixture mismatch')
    require(own.get('iss') != issuer.get('iss') and own.get('tenant_id') == issuer.get('tenant_id'), 'foreign issuer fixture mismatch')
    for principal in foreign:
        host.api('GET', '/api/memory/policy', principal=principal)
    host.api('GET', '/api/memory/policy', principal='other_issuer', expect=401)
    session = str(uuid.uuid4())
    marker = 'owner-isolation-' + uuid.uuid4().hex
    run = host.api('POST', '/api/uar/runs', {'artifact': spec['artifact'], 'session_id': session,
        'input': tool_input(marker=marker), 'run_credentials': spec['run_credentials']})['run_id']
    approval = poll(lambda: pending(host, run), bool, 'required approval absent')
    approval_id = approval.get('approval_id') or approval['approvalId']
    prefix = '/api/uar/runs/' + segment(run)
    for suffix in ('', '/events', '/stream', '/checkpoints', '/tool-admission-evidence'):
        host.api('GET', prefix + suffix, expect=401, principal='other_issuer')
    host.api('POST', prefix + '/tool-approval', {'approval_id': approval_id, 'approved': True},
             expect=401, principal='other_issuer')
    host.api('GET', '/api/uar/sessions/' + session + '/context-stats')
    for principal in foreign:
        for suffix in ('', '/events', '/stream', '/checkpoints', '/tool-admission-evidence'):
            host.api('GET', prefix + suffix, expect=(403, 404), principal=principal)
        require(host.api('GET', prefix + '/tool-approval/pending', principal=principal)['pending'] is None,
                'foreign principal received pending approval')
        require(not host.api('GET', prefix + '/tool-approval', principal=principal)['records'],
                'foreign principal received durable approval')
        host.api('POST', prefix + '/tool-approval', {'approval_id': approval_id, 'approved': True},
                 expect=(403, 404), principal=principal)
        host.api('POST', prefix + '/cancel', {}, expect=(403, 404), principal=principal)
        host.api('POST', prefix + '/resume', {}, expect=(403, 404), principal=principal)
        require(not host.api('POST', '/api/uar/sessions/' + session + '/cancel', {}, principal=principal)['cancelled'],
                'foreign session cancellation succeeded')
        host.api('GET', '/api/uar/sessions/' + session + '/context-stats', expect=404, principal=principal)
        host.api('DELETE', '/api/uar/sessions/' + session, expect=404, principal=principal)
    still = pending(host, run)
    require((still.get('approval_id') or still['approvalId']) == approval_id, 'foreign calls changed live approval')
    decision(host, run, True)
    finish(host, run)
    evidence.observe('owner-thread-isolation', {'run_id': run, 'session_id': session,
        'approval_id': approval_id, 'foreign_principals': list(foreign), 'owner_completed': True})
    evidence.observe('owner-untrusted-issuer-rejection', {'run_id': run, 'principal': 'other_issuer',
        'status': 401, 'claim_scope': 'authentication rejection; not authenticated issuer isolation'})

    policy_path = '/api/uar/conversations/' + session + '/policy'
    saved = host.api('PUT', policy_path, {'memory_enabled': False})
    own_policy = host.api('GET', policy_path)
    for principal in foreign:
        require(host.api('GET', policy_path, principal=principal) is None, 'legacy policy leaked across principal')
        host.api('DELETE', policy_path, expect=204, principal=principal)
        require(host.api('GET', policy_path) == own_policy, 'foreign reset changed owner policy')
        host.api('GET', '/api/uar/sessions/' + session + '/agent-config', expect=204, principal=principal)
        host.api('POST', '/api/uar/sessions/' + session + '/agent-config',
                 {'agent_id': spec['artifact']['id'], 'knowledge_bases': []}, principal=principal)
        require(host.api('GET', policy_path) == own_policy, 'foreign agent config overwrote owner policy')
    config = json.loads(Path(host.spec['config']).read_text())
    acp_path = config.get('acp', {}).get('path', '/acp')
    def rpc(method, params, principal='owner'):
        return host.api('POST', acp_path, {'jsonrpc': '2.0', 'id': str(uuid.uuid4()),
                                        'method': method, 'params': params}, principal=principal)
    acp = rpc('sessions/create', {})['result']['session_id']
    for principal in foreign:
        require('error' in rpc('sessions/get', {'session_id': acp}, principal), 'ACP session disclosed')
        require('error' in rpc('runs/create', {'session_id': acp, 'input': 'foreign'}, principal), 'ACP foreign dispatch admitted')
        require('error' in rpc('runs/get', {'run_id': run}, principal), 'ACP run disclosed')
        require(not rpc('sessions/delete', {'session_id': acp}, principal)['result']['deleted'], 'ACP foreign deletion succeeded')
    require(rpc('sessions/get', {'session_id': acp})['result']['session_id'] == acp, 'ACP owner lost session')
    rpc('sessions/delete', {'session_id': acp})
    evidence.observe('owner-adjacent-isolation', {'session_id': session, 'acp_session_id': acp,
        'policy_owner': saved['owner_id'], 'foreign_principals': list(foreign)})

    # The completed real run supplies accounting and admission evidence. Erasing
    # its conversation must not erase independently governed host records.
    accounting_path = '/api/uar/accounting/calls/' + segment(run)
    audit_path = prefix + '/tool-admission-evidence'
    accounting = host.api('GET', accounting_path)
    audit = host.api('GET', audit_path)
    require(accounting and audit['records'], 'real run accounting/admission retention controls absent')
    foreign_policies = {principal: host.api('GET', policy_path, principal=principal) for principal in foreign}
    require(all(foreign_policies.values()), 'foreign policy retention controls absent')
    host.api('DELETE', '/api/uar/sessions/' + session, expect=204)
    host.api('GET', '/api/uar/sessions/' + session + '/context-stats', expect=404)
    require(host.api('GET', policy_path) is None, 'erased session retained its policy')
    require(host.api('GET', accounting_path) == accounting, 'session erasure changed independent accounting')
    require(host.api('GET', audit_path) == audit, 'session erasure changed independent admission evidence')
    for principal in foreign:
        require(host.api('GET', policy_path, principal=principal) == foreign_policies[principal],
                'session erasure removed another principal policy')
    evidence.observe('session-erasure-authority-scope', {'run_id': run, 'session_id': session,
        'accounting_calls_retained': len(accounting), 'admission_records_retained': len(audit['records']),
        'foreign_principals': list(foreign), 'own_policy_removed': True})


def run_unresolved_knowledge(host, spec, evidence, provider):
    """Actual worker upload, positive retrieval control, then unresolved selection."""
    secret = 'UNSELECTED_KB_SECRET_' + uuid.uuid4().hex
    kb = host.api('POST', '/api/uar/knowledge-bases/', {'name': 'scope-' + uuid.uuid4().hex,
                  'config': {'retrieval_min_score': 0.0}}, expect=201)
    base = '/api/uar/knowledge-bases/' + segment(kb['id'])
    boundary = 'dot-owner-' + uuid.uuid4().hex
    body = (f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="scope.txt"\r\n'
            f'Content-Type: text/plain\r\n\r\n{secret}\r\n--{boundary}--\r\n').encode()
    doc = host.api('POST', base + '/documents', body, expect=202,
                   headers={'Content-Type': 'multipart/form-data; boundary=' + boundary})
    poll(lambda: host.api('GET', base + '/documents/' + segment(doc['id']) + '/ingestion'),
         lambda row: row['state'] == 'indexed', 'real scoped upload not indexed', 300)
    require(any(secret in row['content'] for row in host.api('POST', base + '/search',
        {'query': secret, 'min_score': 0.0})['results']), 'positive indexed search control failed')
    observed = []
    for selection, expect_secret in [(kb['name'], True), ('missing-' + uuid.uuid4().hex, False)]:
        artifact = copy.deepcopy(spec['artifact'])
        artifact.setdefault('memory', {})['kb'] = {'enabled': True, 'knowledge_bases': [selection], 'citation_required': True}
        request_marker = 'scope-request-' + uuid.uuid4().hex
        run = host.api('POST', '/api/uar/runs', {'artifact': artifact,
            'input': json.dumps({'dot_action': 'text', 'marker': request_marker, 'query': secret}),
            'run_credentials': spec['run_credentials']})['run_id']
        finish(host, run)
        requests = provider.rows()
        requests = [row for row in requests if row.get('instruction', {}).get('marker') == request_marker]
        require(requests, 'actual provider call absent')
        received = any(secret in json.dumps([message for message in row['messages'] if message['role'] != 'user']) for row in requests)
        require(received == expect_secret, 'unresolved selection broadened search or positive RAG control failed')
        observed.append({'run_id': run, 'selection': selection, 'received_private_chunk': received})
    host.api('DELETE', base, expect=(200, 204))
    evidence.observe('knowledge-unresolved-selection', {'kb_id': kb['id'], 'runs': observed})
