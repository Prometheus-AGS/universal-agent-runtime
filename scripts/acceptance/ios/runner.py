"""Explicitly authorized simulator preparation/build/execution only. Never import to run gates."""
from __future__ import annotations
import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import selectors
import signal
import math
import sqlite3
import subprocess
import sys
import time
import uuid

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
BUNDLE = "org.prometheus.uar.acceptance"
DEADLINE = 0.0

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def digest(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()

def read(path):
    return json.loads(Path(path).read_bytes())

def write(path, value):
    with os.fdopen(os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600), "w") as output:
        json.dump(value, output, sort_keys=True)
        output.flush()
        os.fsync(output.fileno())

def command(args, *, log=None, allow_failure=False):
    remaining = DEADLINE - time.monotonic()
    require(remaining > 0,"overall command allowance exhausted")
    command_deadline = min(DEADLINE,time.monotonic() + (7200 if log else 120))
    limit = 16_777_216 if log else 4_194_304
    output = open(log,"xb") if log else None
    process = subprocess.Popen(args,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout,selectors.EVENT_READ)
    raw = bytearray()
    observed = 0
    try:
        while selector.get_map():
            require(time.monotonic() < command_deadline,"owned command timed out")
            for key,_ in selector.select(timeout=min(0.25,max(0,command_deadline-time.monotonic()))):
                chunk = os.read(key.fileobj.fileno(),65536)
                if not chunk:
                    selector.unregister(key.fileobj)
                    continue
                observed += len(chunk)
                require(observed <= limit,"owned command output exceeded bound")
                if output:
                    output.write(chunk)
                else:
                    raw.extend(chunk)
        process.wait(timeout=max(0.001,command_deadline-time.monotonic()))
        require(allow_failure or process.returncode==0,"owned command failed; inspect private log")
        return (process.returncode,bytes(raw)) if allow_failure else bytes(raw)
    finally:
        selector.close()
        # Signal only the process group created for this exact command, including
        # descendants that retained a pipe after the original command exited.
        try:
            os.killpg(process.pid,signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            pass
        finally:
            try:
                os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=2)
        process.stdout.close()
        if output:
            output.close()

def metadata():
    devices = json.loads(command(["xcrun", "simctl", "list", "--json", "devices", "available"]))
    runtimes = json.loads(command(["xcrun", "simctl", "list", "--json", "runtimes"]))
    sdk_path = command(["xcrun", "--sdk", "iphonesimulator", "--show-sdk-path"]).decode().strip()
    sdk_version = command(["xcrun", "--sdk", "iphonesimulator", "--show-sdk-version"]).decode().strip()
    sdk = read(Path(sdk_path) / "SDKSettings.json")
    architectures = sdk["SupportedTargets"]["iphonesimulator"]["Archs"]
    arch = platform.machine()
    require(arch in architectures and arch in ("arm64", "x86_64"), "host architecture not supported by installed simulator SDK")
    return devices, runtimes, {"sdk_path":sdk_path,"sdk_version":sdk_version,"arch":arch,
        "target":"aarch64-apple-ios-sim" if arch == "arm64" else "x86_64-apple-ios",
        "xcode":command(["xcodebuild", "-version"]).decode().strip(),
        "developer_dir":command(["xcode-select", "-p"]).decode().strip()}

def prepare(args):
    devices, runtimes, selected = metadata()
    choices = []
    for runtime in runtimes["runtimes"]:
        if not runtime.get("isAvailable") or not runtime["identifier"].startswith("com.apple.CoreSimulator.SimRuntime.iOS-"):
            continue
        for device in devices["devices"].get(runtime["identifier"], []):
            if device.get("isAvailable") and device.get("deviceTypeIdentifier", "").startswith("com.apple.CoreSimulator.SimDeviceType.iPhone-") and (args.udid is None or device["udid"] == args.udid):
                choices.append((tuple(int(v) for v in runtime["version"].split(".")),runtime["identifier"],device["udid"],runtime,device))
    require(bool(choices), "no available installed iOS simulator")
    newest = max(row[0] for row in choices)
    choice = sorted((row for row in choices if row[0] == newest),key=lambda row:(row[1],row[2]))[0]
    selected.update({"schema_version":1,"runtime":choice[3],"device":choice[4],"udid":choice[2]})
    write(args.selection, selected)

def frozen(selection):
    devices, runtimes, current = metadata()
    require(all(selection[key] == current[key] for key in current), "frozen SDK/Xcode/architecture changed")
    require(any(row["identifier"] == selection["runtime"]["identifier"] and row.get("isAvailable") and row["version"] == selection["runtime"]["version"] and row["buildversion"] == selection["runtime"]["buildversion"] for row in runtimes["runtimes"]), "frozen runtime unavailable or changed")
    require(any(row["udid"] == selection["udid"] and row.get("isAvailable") for row in devices["devices"].get(selection["runtime"]["identifier"], [])), "frozen simulator unavailable")

def tree_identity(path):
    rows = []
    for item in sorted(path.rglob("*")):
        require(time.monotonic() < DEADLINE,"artifact inspection allowance exhausted")
        if item.is_file():
            rows.append({"path":str(item.relative_to(path)),"sha256":digest(item.read_bytes())})
    return {"sha256":digest(json.dumps(rows,sort_keys=True,separators=(",",":")).encode()),"files":rows}

def source_rows(checkout, visited):
    resolved = checkout.resolve()
    if resolved in visited:
        return
    visited.add(resolved)
    sys.path.insert(0,str(HERE.parent))
    import source_inventory
    raw = command(["git","-C",str(checkout),"ls-files","-z","--cached","--others","--exclude-standard"])
    for name in sorted(set(os.fsdecode(item) for item in raw.split(b"\0") if item)):
        require(time.monotonic() < DEADLINE,"source inspection allowance exhausted")
        path = checkout/name
        if not source_inventory.selected(path.relative_to(ROOT)):
            continue
        if path.is_symlink():
            yield {"path":str(path),"kind":"symlink","target":os.readlink(path),"resolved_path":str(path.resolve()),"sha256":digest(path.read_bytes()) if path.is_file() else None}
        elif path.is_file():
            raw_file = path.read_bytes()
            yield {"path":str(path),"kind":"regular","bytes":len(raw_file),"sha256":digest(raw_file)}
        elif path.is_dir():
            yield from source_rows(path,visited)
        else:
            yield {"path":str(path),"kind":"missing","sha256":None}

def source_snapshot(path):
    inventory = read(path)
    require(Path(inventory["root"]).resolve() == ROOT,"source inventory root differs")
    current = sorted(source_rows(ROOT,set()),key=lambda item:item["path"])
    require(current == inventory["files"],"source closure changed, including additions/symlinks/deletions")
    return inventory

def build(args):
    selection = read(args.selection)
    inventory = source_snapshot(args.source_inventory)
    frozen(selection)
    output = Path(args.output).resolve()
    output.mkdir(mode=0o700)
    command(["cargo","build","--locked","--manifest-path",str(ROOT/"sdks/rust/Cargo.toml"),"--no-default-features","--features","ios-acceptance","--example","ios_acceptance","--target",selection["target"],"--target-dir",str(output/"rust")],log=output/"cargo.log")
    library = output/"rust"/selection["target"] / "debug/examples/libios_acceptance.a"
    command(["xcodebuild","build-for-testing","-project",str(HERE/"Acceptance.xcodeproj"),"-scheme","UARAcceptance","-configuration","Debug","-destination","id="+selection["udid"],"-derivedDataPath",str(output/"xcode"),"ARCHS="+selection["arch"],"UAR_RUST_LIBRARY="+str(library),"CODE_SIGNING_ALLOWED=NO"],log=output/"xcode-build.log")
    products = output/"xcode/Build/Products"
    write(output/"artifact.json",{"schema_version":1,"selection":selection,"source_identity":inventory["source_identity"],"source_inventory_sha256":digest(Path(args.source_inventory).read_bytes()),"staticlib_sha256":digest(library.read_bytes()),"products":tree_identity(products)})

def ledger_snapshot(config, remaining=None):
    deadline = DEADLINE if remaining is None else min(DEADLINE,time.monotonic()+remaining)
    def allowance():
        seconds = deadline-time.monotonic()
        require(seconds > 0,"ledger snapshot allowance exhausted")
        return seconds
    sys.path.insert(0,str(HERE.parent))
    import budget_proxy
    approved, binding, ready = budget_proxy.owned_hop(config["budget_proxy_config"],config["base_url"])
    path = Path(approved["ledger_root"])/"ledger.sqlite3"
    with closing(sqlite3.connect(path.as_uri()+"?mode=ro",uri=True,
                                timeout=min(5.0,allowance()))) as db:
        db.set_progress_handler(lambda: int(time.monotonic() >= deadline),1000)
        db.execute("PRAGMA busy_timeout="+str(int(min(5.0,allowance())*1000)))
        meta = {key:json.loads(value) for key,value in db.execute("SELECT key,value FROM meta")}
        db.execute("PRAGMA busy_timeout="+str(int(min(5.0,allowance())*1000)))
        rows = {key:{"state":state,"receipt":json.loads(receipt),"reserved":json.loads(reserved)}
                for key,state,receipt,reserved in db.execute("SELECT request_id,state,receipt,reserved FROM reservations")}
    require(meta["blocked"] is None,"budget epoch has unknown accounting")
    require(all(meta["totals"][key] <= approved["limits"][key] for key in budget_proxy.COUNTERS),"cumulative epoch limit exceeded")
    snapshot = {"model_output_cap":binding["max_output_tokens_per_attempt"],"model_response_cap":binding["max_response_bytes"],"deadline_unix":read(approved["seed"]["path"])["deadline_unix"],"epoch_id":approved["epoch_id"],"input_identity":ready["epoch"]["input_identity"],"binding_id":binding["id"],"totals":meta["totals"],"reservations":rows}
    allowance()
    return snapshot

def item_manifest(run_id):
    return [{"backend":"ios", "case_id":phase, "attempt":repetition,
             "item_id":f"ios:{phase}:{repetition}", "sweep_run_id":run_id,
             "dependency_ids":[f"ios:seed:{repetition}"] if phase == "restore" else [],
             "status":"NOT_RUN"}
            for repetition in range(1,4) for phase in ("seed","restore")]

def collection_status(rows):
    identities = {(row.get("source_identity"),row.get("artifact_identity"))
                  for row in rows if row["status"] == "PASS"}
    if len(identities) > 1:
        return "BLOCKED"
    if any(row["status"] == "FAIL" for row in rows):
        return "FAIL"
    return "PASS" if len(rows) == 6 and all(row["status"] == "PASS" for row in rows) else "BLOCKED"

def sealed_file(reference):
    path = Path(reference["path"])
    require(path.is_absolute() and path == path.resolve() and path.is_file()
            and path.stat().st_size <= 64*1024*1024
            and digest(path.read_bytes()) == reference["sha256"],"frozen correction input differs")
    return path

def owned_tree(path):
    require(not path.is_symlink(),"owned state is a symlink")
    if not path.exists():
        return {"state":"absent"}
    require(path.is_dir() and all(not item.is_symlink() for item in path.rglob("*")),
            "owned state has unexpected shape")
    size, count = 0, 0
    for item in path.rglob("*"):
        require(time.monotonic() < DEADLINE,"owned baseline allowance exhausted")
        count += 1
        if item.is_file():
            size += item.stat().st_size
            require(item.stat().st_size <= 16_777_216,"owned baseline file exceeds bound")
        require(count <= 10000 and size <= 67_108_864,"owned baseline exceeds retention bound")
    return {"state":"directory", "identity":tree_identity(path)}

def settled_ledger(config, before):
    sys.path.insert(0,str(HERE.parent))
    import budget_proxy
    deadline = min(DEADLINE,time.monotonic()+90)
    while True:
        remaining = min(deadline,DEADLINE)-time.monotonic()
        require(remaining > 0,"forwarding settlement unknown; stop dispatch")
        after = ledger_snapshot(config,remaining=remaining)
        require(time.monotonic() < min(deadline,DEADLINE),"forwarding settlement unknown; stop dispatch")
        require(all(after[key] == before[key] for key in
                    ("epoch_id","input_identity","binding_id","model_output_cap","model_response_cap")),"budget binding changed")
        pending = [row for row in after["reservations"].values() if row["state"] == "reserved"]
        if not pending:
            for key,row in after["reservations"].items():
                if key in before["reservations"]:
                    continue
                receipt, reserved = row["receipt"], row.get("reserved")
                require(isinstance(receipt,dict),"new forwarding receipt malformed")
                require(isinstance(reserved,dict)
                        and all(type(reserved.get(name)) is int and reserved[name] >= 0
                                for name in ("generated_tokens","response_bytes")),
                        "new forwarding accounting unknown")
                error = budget_proxy.terminal_receipt_error(row["state"],receipt,
                    epoch_id=after["epoch_id"],input_identity=after["input_identity"],
                    binding_id=after["binding_id"],kind="model",
                    max_response_bytes=min(reserved["response_bytes"],after["model_response_cap"]),
                    max_output_tokens=min(reserved["generated_tokens"],after["model_output_cap"]))
                require(error != "accounting","new forwarding accounting unknown")
                require(error != "usage","new successful forwarding usage unknown")
            require(time.monotonic() < min(deadline,DEADLINE),"forwarding settlement unknown; stop dispatch")
            return after
        remaining = min(deadline,DEADLINE)-time.monotonic()
        require(remaining > 0,"forwarding settlement unknown; stop dispatch")
        time.sleep(min(0.25,remaining))

def execute(args):
    global DEADLINE
    selection = read(args.selection)
    inventory = source_snapshot(args.source_inventory)
    frozen(selection)
    output = Path(args.output).resolve()
    results = Path(args.results)
    require(results.is_absolute() and results == results.resolve(),"fresh result scope must be absolute and direct")
    results.mkdir(mode=0o700)
    artifact = read(output/"artifact.json")
    require(artifact["selection"] == selection,"artifact target differs")
    require(artifact["source_identity"] == inventory["source_identity"],"artifact source identity differs")
    require(artifact["source_inventory_sha256"] == digest(Path(args.source_inventory).read_bytes()),"source inventory differs")
    require(artifact["products"] == tree_identity(output/"xcode/Build/Products"),"built test products changed")
    config_path = Path(args.private_config)
    require(config_path.stat().st_mode & 0o077 == 0,"private configuration permissions must be0600")
    config = read(config_path)
    before = ledger_snapshot(config)
    require(type(config.get("context_window")) is int and config["context_window"] > before["model_output_cap"],"explicit host-admitted model context must exceed proxy output ceiling")
    require(before["deadline_unix"] > time.time(),"approved epoch allowance exhausted")
    DEADLINE = min(DEADLINE,time.monotonic()+before["deadline_unix"]-time.time())
    udid = selection["udid"]
    # Boot is an execution action, never part of target selection.
    all_devices = json.loads(command(["xcrun","simctl","list","--json","devices"]))
    device = next(row for rows in all_devices["devices"].values() for row in rows if row["udid"]==udid)
    if device["state"] != "Booted":
        command(["xcrun","simctl","boot",udid])
    command(["xcrun","simctl","bootstatus",udid,"-b"])
    app = output/"xcode/Build/Products/Debug-iphonesimulator/UARAcceptance.app"
    existing_code, _ = command(["xcrun","simctl","get_app_container",udid,BUNDLE,"data"],allow_failure=True)
    require(existing_code != 0,"existing installed harness app is unowned; preserve it and select a fresh simulator")
    command(["xcrun","simctl","install",udid,str(app)])
    container = Path(command(["xcrun","simctl","get_app_container",udid,BUNDLE,"data"]).decode().strip())
    documents = container/"Documents"
    documents.mkdir(exist_ok=True)
    require(not any(documents.iterdir()),"existing harness app data is unowned; preserve it and use a fresh operator-owned simulator")
    marker = {"schema_version":1,"scope_id":config["scope_id"],"source":artifact["source_identity"],"artifact":artifact["products"]["sha256"],"epoch_id":before["epoch_id"],"udid":udid}
    write(documents/"ownership.json",marker)
    policy_path = documents/"policies"
    shutil.copytree(HERE/"policies",policy_path)
    # Result scopes are exclusive and separate from immutable built products.
    write(results/"run-ownership.json",marker)
    sys.path.insert(0,str(HERE.parent))
    from core import Evidence
    from failure_sweep import await_correction, private_json
    control = private_json({"path":str(Path(args.collection_control).resolve()),
                            "sha256":digest(Path(args.collection_control).read_bytes())}) if args.collection_control else {
        "run_id":str(uuid.uuid4()),"corrections_before":0,"max_corrections":0,"immutable_files":[]}
    require(type(control["corrections_before"]) is int and type(control["max_corrections"]) is int
            and 0 <= control["corrections_before"] <= control["max_corrections"],"shared repair counters invalid")
    require(control.get("epoch_id",before["epoch_id"]) == before["epoch_id"],"collection epoch differs")
    immutable = control["immutable_files"] + [{"path":str(Path(__file__).resolve()),
                                               "sha256":digest(Path(__file__).read_bytes())},
        {"path":str(config_path.resolve()),"sha256":digest(config_path.read_bytes())},
        {"path":str(Path(args.selection).resolve()),"sha256":digest(Path(args.selection).read_bytes())}]
    if args.collection_control:
        immutable.append({"path":str(Path(args.collection_control).resolve()),
                          "sha256":digest(Path(args.collection_control).read_bytes())})
    authority = None
    if control.get("correction_authority") is not None:
        authority = private_json(control["correction_authority"])
        immutable.append(control["correction_authority"])
        root = Path(authority.get("receipt_directory",""))
        require(authority.get("schema_version") == 1 and authority.get("operator_approved") is True
                and authority.get("operation") == "correct-uar-ios-item"
                and isinstance(authority.get("authority_id"),str) and bool(authority["authority_id"])
                and authority.get("source_identity") == artifact["source_identity"]
                and authority.get("artifact_identity") == artifact["products"]["sha256"]
                and authority.get("epoch_id") == before["epoch_id"]
                and type(authority.get("max_corrections")) is int and 0 < authority["max_corrections"] <= 6
                and type(authority.get("wait_seconds")) in (int,float)
                and math.isfinite(authority["wait_seconds"]) and 0 < authority["wait_seconds"] <= 3600
                and root.is_absolute() and root == root.resolve() and root.is_dir()
                and root.stat().st_uid == os.getuid() and root.stat().st_mode & 0o077 == 0,
                "actual bounded iOS correction authority required")
    events = Evidence(results/"events",[])
    rows = item_manifest(control["run_id"])
    indexed = {row["item_id"]:row for row in rows}
    receipts, corrections = [], 0
    safety_stop = False
    active_output, active_inventory = output, Path(args.source_inventory)
    private_path = documents/"private-config.json"

    def remaining():
        require(time.monotonic() < DEADLINE,"shared time exhausted")
        for reference in immutable:
            sealed_file(reference)
        settled_ledger(config,before)
        require(read(documents/"ownership.json") == marker,"app ownership changed")
        current = Path(command(["xcrun","simctl","get_app_container",udid,BUNDLE,"data"]).decode().strip())
        require(current == container,"owned app container changed")
        return DEADLINE-time.monotonic()

    def attempt(row, revision):
        remaining()
        attempt_before = settled_ledger(config,before)
        row.pop("reason",None)
        require(source_snapshot(active_inventory)["source_identity"] == artifact["source_identity"],"source rebind differs")
        require(artifact["products"] == tree_identity(active_output/"xcode/Build/Products"),"products changed")
        require(tree_identity(policy_path) == tree_identity(HERE/"policies"),"mounted policy source differs")
        installed = Path(command(["xcrun","simctl","get_app_container",udid,BUNDLE,"app"]).decode().strip())
        require(installed.is_absolute() and installed == installed.resolve()
                and tree_identity(installed) == tree_identity(active_output/"xcode/Build/Products/Debug-iphonesimulator/UARAcceptance.app"),
                "installed owned app differs from admitted product")
        scope = results/(row["case_id"]+"-"+str(row["attempt"])+"-attempt-"+str(revision))
        scope.mkdir(mode=0o700)
        store = documents/("store-"+str(row["attempt"]))
        baseline = {"path":str(store),"state":owned_tree(store)}
        if baseline["state"]["state"] == "directory":
            snapshot = scope/"store-baseline"
            shutil.copytree(store,snapshot)
            require(owned_tree(snapshot) == baseline["state"],"owned store changed during baseline retention")
            baseline["retained_snapshot"] = str(snapshot)
        write(scope/"baseline.json",baseline)
        private = {key:config[key] for key in ("base_url","proxy_token","model","context_window")}
        private.update({"model_output_cap":before["model_output_cap"],"epoch_id":before["epoch_id"],
            "source":artifact["source_identity"],"artifact":artifact["products"]["sha256"],
            "selection":selection,"repetition":row["attempt"]})
        native = documents/(row["case_id"]+"-"+str(row["attempt"])+".json")
        require(not native.exists(),"prior native receipt must be retained before same-item rerun")
        write(private_path,private)
        private_digest = digest(private_path.read_bytes())
        try:
            test = "testSeed" if row["case_id"] == "seed" else "testRestore"
            code, _ = command(["xcodebuild","test-without-building","-project",str(HERE/"Acceptance.xcodeproj"),
                "-scheme","UARAcceptance","-configuration","Debug","-destination","id="+udid,
                "-derivedDataPath",str(active_output/"xcode"),"-parallel-testing-enabled","NO",
                "-maximum-concurrent-test-simulator-destinations","1",
                "-only-testing:UARAcceptanceTests/EmbeddedAcceptance/"+test,
                "-resultBundlePath",str(scope/"result.xcresult"),"CODE_SIGNING_ALLOWED=NO"],
                log=scope/"xcode.log",allow_failure=True)
            row.update(status="FAIL" if code else "PASS",returncode=code,revision_attempt=revision,
                       source_identity=artifact["source_identity"],artifact_identity=artifact["products"]["sha256"])
            write(scope/"invocation-result.json",dict(row))
            if native.exists():
                require(not native.is_symlink() and native.is_file(),"native evidence ownership unknown")
                raw = native.read_bytes()
                with os.fdopen(os.open(scope/"native-original.json",os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600),"wb") as retained:
                    retained.write(raw)
                try:
                    receipt = json.loads(raw)
                except (ValueError,UnicodeError):
                    receipt = None
                identity_matches = (isinstance(receipt,dict)
                        and receipt.get("source") == artifact["source_identity"]
                        and receipt.get("artifact") == artifact["products"]["sha256"]
                        and receipt.get("selection") == selection and receipt.get("phase") == row["case_id"]
                        and receipt.get("repetition") == row["attempt"])
                if identity_matches:
                    write(scope/"native.json",receipt)
                else:
                    row.update(status="FAIL",reason="native_receipt_malformed_or_identity_differs")
                if code == 0 and identity_matches:
                    receipts.append(receipt)
            elif code == 0:
                row.update(status="FAIL",reason="successful_process_missing_native_receipt")
        finally:
            require(read(documents/"ownership.json") == marker and not private_path.is_symlink()
                    and digest(private_path.read_bytes()) == private_digest,"private input cleanup ownership unknown")
            private_path.unlink()
            # A failed terminate alone never proves owned runtime/store cleanup.
            termination_code, termination = command(["xcrun","simctl","terminate",udid,BUNDLE],allow_failure=True)
            write(scope/"termination.json",{"returncode":termination_code,"output":termination.decode(errors="replace")})
            # XCTest's process cleanup is not inferred from a failed terminate command.
            close = read(scope/"native.json").get("shutdown",{}) if (scope/"native.json").exists() else {}
            require(termination_code == 0 or (close.get("runtime_released") is True
                    and close.get("store_released") is True and close.get("engine_acknowledged") is True
                    and close.get("runtime_failures") == [] and close.get("unresolved") == []
                    and close.get("store_failure") is None),
                    "owned app termination unconfirmed")
        remaining()
        attempt_after = settled_ledger(config,attempt_before)
        if any(value["state"] == "failed" for key,value in attempt_after["reservations"].items()
               if key not in attempt_before["reservations"]):
            row.update(status="FAIL",reason="model_forwarding_failed")
        require(source_snapshot(active_inventory)["source_identity"] == artifact["source_identity"]
                and artifact["products"] == tree_identity(active_output/"xcode/Build/Products"),
                "source or product changed during invocation")
        require(tree_identity(installed) == tree_identity(active_output/"xcode/Build/Products/Debug-iphonesimulator/UARAcceptance.app"),
                "installed app changed during invocation")
        row["cleanup_confirmed"] = True
        write(scope/"terminal.json",dict(row))
        failure = {"item_id":row["item_id"],"revision_attempt":revision,
                   "receipt":{"path":str(scope/"terminal.json"),"sha256":digest((scope/"terminal.json").read_bytes())}}
        return failure, baseline, native, scope

    try:
        for row in rows:
            remaining()
            dependencies = [indexed[name] for name in row["dependency_ids"]]
            if any(value["status"] != "PASS" for value in dependencies):
                row.update(status="BLOCKED",reason="same_repetition_seed_not_passed")
                events.write("item-blocked",dict(row))
                continue
            failure, baseline, native, scope = attempt(row,0)
            if row["status"] == "PASS":
                continue
            events.write("initial-failure",failure)
            if (authority is not None and corrections < authority["max_corrections"]
                    and control["corrections_before"]+corrections < control["max_corrections"]):
                old = {"source_identity":artifact["source_identity"],"artifact_identity":artifact["products"]["sha256"]}
                events.write("correction-baseline",{"item_id":row["item_id"],"baseline":baseline,
                    "ownership":marker,"native_receipt":str(native),"selection":selection})
                receipt = await_correction(authority,row,failure,old,events,remaining)
                if receipt is not None:
                    # Charge first: failed refresh admission still consumed actual host work.
                    corrections += 1
                    write(results/("correction-consumed-"+str(corrections)+".json"),{
                        "run_id":control["run_id"],"item_id":row["item_id"],"receipt":receipt,
                        "shared_corrections_consumed":control["corrections_before"]+corrections})
                    require(receipt.get("approved_effects") == authority.get("approved_effects")
                            and authority.get("approved_effects") == {
                                "refresh_source_inventory":True,"build_fresh_artifact":True,
                                "install_owned_app_preserving_container":True,"restore_exact_owned_store_baseline":True}
                            and receipt.get("owned_baseline_restored") == baseline
                            and owned_tree(Path(baseline["path"])) == baseline["state"],
                            "host did not admit and restore exact owned pre-item state")
                    for reference in immutable:
                        sealed_file(reference)
                    refreshed = receipt["refreshed_ios"]
                    require(refreshed["artifact_root"] in authority.get("refreshable_artifact_roots",[])
                            and refreshed["source_inventory"]["path"] in authority.get("refreshable_inventory_paths",[]),
                            "refreshed artifact/inventory path not authorized")
                    active_inventory = sealed_file(refreshed["source_inventory"])
                    inventory = source_snapshot(active_inventory)
                    next_output = Path(refreshed["artifact_root"])
                    require(next_output.is_absolute() and next_output == next_output.resolve()
                            and next_output != active_output,"correction needs a distinct direct artifact root")
                    active_output = next_output
                    artifact = read(active_output/"artifact.json")
                    require(artifact["source_identity"] == inventory["source_identity"] == receipt["new_source_identity"]
                            and artifact["products"]["sha256"] == receipt["new_artifact_identity"]
                            and artifact["source_identity"] != old["source_identity"]
                            and artifact["products"]["sha256"] != old["artifact_identity"]
                            and artifact["selection"] == selection
                            and artifact["source_inventory_sha256"] == digest(active_inventory.read_bytes())
                            and artifact["products"] == tree_identity(active_output/"xcode/Build/Products"),
                            "actual corrected source/product rebind failed")
                    remaining()
                    if native.exists():
                        # Retain before removing only the exact unchanged owned receipt.
                        raw = native.read_bytes()
                        require(not native.is_symlink() and raw == (scope/"native-original.json").read_bytes(),
                                "native receipt ownership changed; preserve it")
                        native.unlink()
                    attempt(row,1)
            if row["status"] != "PASS":
                events.write("unresolved",dict(row))
    except Exception as error:
        safety_stop = True
        events.write("safety-stop",{"error_type":type(error).__name__})
    after = settled_ledger(config,before)
    status = "BLOCKED" if safety_stop else collection_status(rows)
    new = {key:value for key,value in after["reservations"].items() if key not in before["reservations"]}
    if status == "PASS" and not new:
        status = "BLOCKED"
    write(results/"receipt.json",{"schema_version":1,"platform":"ios-simulator","status":status,
        "run_id":control["run_id"],"source":marker["source"],"artifact":marker["artifact"],
        "final_source_identity":artifact["source_identity"],"final_artifact_identity":artifact["products"]["sha256"],
        "selection":selection,"repetitions":3,"inventory":rows,"native_receipts":receipts,
        "corrections_consumed":corrections,"shared_corrections_consumed":control["corrections_before"]+corrections,
        "safety_stop":safety_stop,"budget_before":before,"budget_after":after,"new_forwarding":new})
    return {"PASS":0,"FAIL":1,"BLOCKED":2}[status]

def main():
    global DEADLINE
    parser = argparse.ArgumentParser()
    parser.add_argument("action",choices=("prepare","build","run"))
    parser.add_argument("--selection",required=True)
    parser.add_argument("--timeout-seconds",required=True,type=float)
    parser.add_argument("--udid")
    parser.add_argument("--output")
    parser.add_argument("--source-inventory")
    parser.add_argument("--private-config")
    parser.add_argument("--results")
    parser.add_argument("--collection-control")
    args=parser.parse_args()
    require(math.isfinite(args.timeout_seconds) and args.timeout_seconds > 0,"finite explicit timeout required")
    maximum = {"prepare":120,"build":7200,"run":7200}[args.action]
    require(args.timeout_seconds <= maximum,"requested timeout exceeds action maximum")
    DEADLINE = time.monotonic()+args.timeout_seconds
    if args.action!="prepare":
        require(args.output and args.source_inventory,"build/run needs output and source inventory")
    if args.action=="run":
        require(args.private_config and args.results,"run needs private configuration and fresh result scope")
    return {"prepare":prepare,"build":build,"run":execute}[args.action](args)

if __name__=="__main__":
    try:
        raise SystemExit(main())
    except Exception:
        # Raw exception strings may contain private endpoints or provider data.
        print("iOS acceptance failed; private logs retained",file=sys.stderr)
        raise SystemExit(1)
