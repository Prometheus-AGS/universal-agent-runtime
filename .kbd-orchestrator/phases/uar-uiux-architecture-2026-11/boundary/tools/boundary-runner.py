import json, os, re, subprocess, sys, time
ROOT = '/Volumes/T7/Projects/prometheus/universal-agent-runtime'
P = ROOT + '/.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'
_contract = json.load(open(P + '/adopted-contract.json'))
cmds = _contract['boundaryPacket']['commands']
CONTRACT = _contract['identity']['contractDigest'][:8]
COMMIT = subprocess.check_output(['git', '-C', ROOT, 'rev-parse', '--short=8', 'HEAD'], text=True).strip()
# argv[1]: a start index ("23") or an explicit ordered list ("0,1,7,8,23-49").
def parse(spec):
    if ',' not in spec and '-' not in spec:
        return list(range(int(spec), len(cmds)))
    out = []
    for part in spec.split(','):
        a, _, b = part.partition('-')
        out += list(range(int(a), int(b or a) + 1))
    return out
order = parse(sys.argv[1]) if len(sys.argv) > 1 else list(range(len(cmds)))
os.makedirs(P + '/boundary/logs', exist_ok=True)
runlog = open(P + '/boundary/run.jsonl', 'a')
ALLOWED = re.compile(r'__eh_frame section too large|generated \d+ warnings?|#\[warn\(linker_messages\)\]')
# Rust/cargo warnings, Vite "(!)" advisories, and rolldown bracketed notices such as [EVAL].
WARN = re.compile(r'^\s*warning\b|\(!\)|^\s*WARN\b|^\s*\[[A-Z][A-Z_]+\]')
ANSI = re.compile(r'\x1b\[[0-9;]*m')
CHUNK = re.compile(r'(\S+\.js)\s+([\d,]+\.\d+) kB')
CHUNK_ADVISORY = 'Some chunks are larger than 1100 kB after minification'
PGLITE = 'node_modules/.pnpm/@electric-sql+pglite@'

def ruled_frontend(lines, k):
    """Operator ruling 2026-11-21: elk-only chunk-size advisory and pglite direct-eval notices."""
    l = lines[k]
    if CHUNK_ADVISORY in l:
        big = [m.group(1) for x in lines for m in [CHUNK.search(x)]
               if m and float(m.group(2).replace(',', '')) > 1100]
        return bool(big) and all(re.search(r'(^|/)elk-[^/]*\.js$', b) for b in big)
    if re.match(r'^\s*\[EVAL\]', l):
        return any(PGLITE in x for x in lines[k + 1:k + 3])
    return False

def warning_lines(lines):
    return [l for k, l in enumerate(lines)
            if WARN.search(l) and not ALLOWED.search(l) and not ruled_frontend(lines, k)]
for i in order:
    c = cmds[i]
    log = f"{P}/boundary/logs/{i:02d}-{c['id']}.log"
    t0 = time.time()
    with open(log, 'w') as fh:
        rc = subprocess.run(c['argv'], cwd=os.path.join(ROOT, c['cwd']), stdout=fh, stderr=subprocess.STDOUT).returncode
    rec = {'index': i, 'id': c['id'], 'kind': c['kind'], 'exit': rc, 'seconds': round(time.time() - t0, 1),
           'commit': COMMIT, 'contract': CONTRACT, 'endedAt': time.strftime('%Y-%m-%dT%H:%M:%S%z')}
    if c['kind'] == 'build' and rc == 0:
        lines = [ANSI.sub('', l) for l in open(log, errors='replace').read().splitlines()]
        bad = warning_lines(lines)
        rec['ruledFrontendNotices'] = sum(1 for k in range(len(lines)) if ruled_frontend(lines, k))
        rec['ruledLinkerLines'] = sum(1 for l in lines if '__eh_frame section too large' in l)
        rec['generatedSummaries'] = [l.strip() for l in lines if re.search(r'generated \d+ warnings?', l)]
        if bad:
            rec['warningStop'] = bad[:20]
    runlog.write(json.dumps(rec) + '\n'); runlog.flush()
    print(json.dumps(rec), flush=True)
    if rc != 0 or rec.get('warningStop'):
        print('STOPPED at', i, c['id'], flush=True); sys.exit(1)
print('COMPLETED all commands', flush=True)
