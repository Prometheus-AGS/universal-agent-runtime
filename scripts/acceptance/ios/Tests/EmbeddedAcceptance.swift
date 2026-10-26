import XCTest
import Foundation

final class EmbeddedAcceptance: XCTestCase {
    enum Failure: Error { case protocolFailure(String) }
    var documents: URL { FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0] }
    func call(_ request: [String: Any]) throws -> Any {
        let data = try JSONSerialization.data(withJSONObject: request, options: [.sortedKeys])
        let text = String(decoding: data, as: UTF8.self)
        guard let result = text.withCString({ uar_ios_call($0) }) else { throw Failure.protocolFailure("null bridge result") }
        defer { uar_ios_free(result) }
        let response = try JSONSerialization.jsonObject(with: Data(String(cString: result).utf8)) as! [String: Any]
        guard response["ok"] as? Bool == true, let value = response["result"] else { throw Failure.protocolFailure("bridge operation refused: \(request["op"] ?? "unknown")") }
        return value
    }
    func object(_ request: [String: Any]) throws -> [String: Any] {
        guard let value = try call(request) as? [String: Any] else { throw Failure.protocolFailure("object required") }
        return value
    }
    func configuration() throws -> [String: Any] {
        try JSONSerialization.jsonObject(with: Data(contentsOf: documents.appendingPathComponent("private-config.json"))) as! [String: Any]
    }
    func open(_ config: [String: Any]) throws {
        var request = config
        request["op"] = "open"
        request["policy_path"] = documents.appendingPathComponent("policies").path
        request["store_path"] = documents.appendingPathComponent("store-\(config["repetition"]!)").path
        let opened = try object(request)
        XCTAssertEqual(opened["driver"] as? String, "LiterLlmDriver")
        XCTAssertEqual(opened["persistence"] as? String, "surrealkv")
        XCTAssertEqual(opened["server"] as? Bool, false)
        XCTAssertEqual(opened["provider_id"] as? String, "openai")
        XCTAssertEqual(opened["wire_model"] as? String, config["model"] as? String)
        XCTAssertEqual(opened["runtime_model"] as? String, "openai/\(config["model"]!)")
    }
    func start(_ agent: String, _ conversation: String, _ prompt: String) throws -> String {
        let response = try object(["op":"start", "agent_id":agent, "conversation":conversation, "prompt":prompt])
        guard let id = response["run_id"] as? String else { throw Failure.protocolFailure("run id missing") }
        return id
    }
    func drain(_ id: String, cancelled: Bool = false) throws -> [[String: Any]] {
        let events = try call(["op":"drain", "run_id":id]) as! [[String: Any]]
        let types = events.compactMap { ($0["event"] as? [String: Any])?["type"] as? String }
        XCTAssertFalse(types.contains("Error"), "canonical error is never success")
        XCTAssertEqual(types.filter { $0 == (cancelled ? "Cancelled" : "RunDone") }.count, 1)
        if !cancelled { XCTAssertFalse(types.contains("Cancelled")) }
        let ids = events.compactMap { $0["id"] as? UInt64 }
        XCTAssertEqual(ids, ids.sorted())
        XCTAssertEqual(Set(ids).count, ids.count)
        return events
    }
    func result(_ events: [[String: Any]]) throws -> NSDictionary {
        let content = events.compactMap { $0["event"] as? [String: Any] }.filter { $0["type"] as? String == "ChatDelta" }
            .compactMap { ($0["data"] as? [String: Any])?["text_delta"] as? String }.joined()
        guard let value = try JSONSerialization.jsonObject(with: Data(content.utf8)) as? NSDictionary else { throw Failure.protocolFailure("strict useful JSON result missing") }
        return value
    }
    func shutdown() throws -> [String: Any] {
        let report = try object(["op":"shutdown"])
        XCTAssertEqual(report["runtime_released"] as? Bool, true)
        XCTAssertEqual(report["store_released"] as? Bool, true)
        XCTAssertEqual(report["engine_acknowledged"] as? Bool, true)
        XCTAssertEqual((report["runtime_failures"] as? [String])?.count, 0)
        XCTAssertEqual((report["unresolved"] as? [String])?.count, 0)
        XCTAssertTrue(report["store_failure"] is NSNull)
        return report
    }
    func save(_ value: [String: Any], _ name: String) throws {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
        try data.write(to: documents.appendingPathComponent(name), options: [.atomic, .completeFileProtectionUntilFirstUserAuthentication])
    }
    func testSeed() throws {
        let config = try configuration()
        try open(config)
        defer { _ = try? call(["op":"shutdown"]) }
        let rep = String(describing: config["repetition"]!)
        let agentID = "ios-invoice-\(rep)"
        var agent = try object(["op":"agent", "id":"default-agent"])
        agent["id"] = agentID
        agent["metadata"] = ["title":"Invoice arithmetic specialist", "description":"Compute invoice line totals, subtotal, currency and discount arithmetic. Route invoice arithmetic here.", "tags":["invoice", "arithmetic"]]
        agent["prompt"] = ["system":"Compute invoice arithmetic exactly. Output only the requested JSON object. Preserve prior conversation facts. Never use tools.","instructions":[]]
        var policy = agent["policy"] as! [String: Any]
        policy["provider"] = ["default":["provider":"openai","model":config["model"]!],"fallbacks":[]]
        policy["tools"] = ["allow":[],"deny":["*"],"max_concurrent":1]
        agent["policy"] = policy
        _ = try call(["op":"upsert_agent", "agent":agent])
        agent = try object(["op":"agent", "id":agentID])
        var global = try object(["op":"get_setting", "key":"run_policy.global"])["value"] as! [String:Any]
        global["context_strategy"] = ["type":"sliding_window","max_messages":16]
        _ = try call(["op":"set_setting", "key":"run_policy.global", "value":global])
        for id in ["default-agent", "orchestrator-agent"] {
            XCTAssertEqual(try object(["op":"delete_agent", "id":id])["refused"] as? Bool, true)
            XCTAssertEqual(try object(["op":"agent", "id":id])["id"] as? String, id)
        }
        let defaultNonce = UUID().uuidString.lowercased()
        let defaultOperand = Int.random(in: 10...999)
        let defaultMultiplier = Int.random(in: 2...9)
        let defaultIncrement = Int.random(in: 1...99)
        let defaultExpected: [String:Any] = ["nonce":defaultNonce,"result":defaultOperand * defaultMultiplier + defaultIncrement]
        let defaultRun = try start("default-agent", "ios-default-seed-\(rep)-\(defaultNonce)", "Compute (\(defaultOperand) multiplied by \(defaultMultiplier)) plus \(defaultIncrement). Return only a JSON object with exactly nonce and result; nonce is \(defaultNonce), result is the computed integer. Do not use tools.")
        let defaultEvents = try drain(defaultRun)
        XCTAssertEqual(try result(defaultEvents), defaultExpected as NSDictionary)
        let defaultChallenge: [String:Any] = ["nonce":defaultNonce,"operands":[defaultOperand,defaultMultiplier,defaultIncrement],"expected":defaultExpected,"run_id":defaultRun,"events":defaultEvents]
        let conversation = "ios-continuity-\(rep)"
        let first = try start(agentID, conversation, "An invoice has 6 items at 7 USD each. Compute the subtotal. Return JSON with exactly subtotal and currency.")
        let firstEvents = try drain(first)
        XCTAssertEqual(try result(firstEvents), ["subtotal":42,"currency":"USD"] as NSDictionary)
        let second = try start(agentID, conversation, "Using the previous invoice subtotal, add shipping of 8 USD. Return JSON with exactly total and currency.")
        let secondEvents = try drain(second)
        XCTAssertEqual(try result(secondEvents), ["total":50,"currency":"USD"] as NSDictionary)
        // Selected manifest must affect a real model response and emit activation.
        let skillID = "ios-tax-\(rep)"
        _ = try call(["op":"skill", "skill":["skill_id":skillID,"version":"1","title":"Tax rule","description":"Calculate ios_tax invoice tax","triggers":["keywords":["ios_tax"]],"prompt_overlay":"For ios_tax, the tax rate is 25 percent. Return JSON with exactly tax and total, calculating both.","enabled":true,"kind":"manifest"]])
        let skillConversation = "ios-skill-\(rep)"
        _ = try call(["op":"policy", "conversation":skillConversation,"policy":["skills":["mode":"selected","ids":[skillID]],"context_strategy":["type":"sliding_window","max_messages":2]]])
        let skilled = try start(agentID, skillConversation, "ios_tax: invoice subtotal is 80. Apply the installed tax rule.")
        let skillEvents = try drain(skilled)
        XCTAssertEqual(try result(skillEvents), ["tax":20,"total":100] as NSDictionary)
        XCTAssertTrue(skillEvents.contains { row in
            guard let event = row["event"] as? [String:Any], let data = event["data"] as? [String:Any] else { return false }
            return event["type"] as? String == "SkillActivated" && data["skill_id"] as? String == skillID
        })
        // Exercise actual structural reduction after enough turns to exceed two messages.
        var contextRun = skilled
        for _ in 0..<2 {
            contextRun = try start(agentID, skillConversation, "ios_tax: a new independent invoice subtotal is 40. Apply the installed tax rule.")
            XCTAssertEqual(try result(drain(contextRun)), ["tax":10,"total":50] as NSDictionary)
        }
        let effective = try object(["op":"effective", "conversation":skillConversation])
        let effectivePolicy = effective["effective_policy"] as! [String: Any]
        XCTAssertEqual((effectivePolicy["context_strategy"] as? [String:Any])?["max_messages"] as? Int, 2)
        let contextRecord = try object(["op":"run", "run_id":contextRun])
        let context = contextRecord["context"] as! [String:Any]
        let reduction = context["context_reduction"] as! [String:Any]
        XCTAssertEqual(reduction["changed"] as? Bool, true)
        XCTAssertGreaterThan(reduction["source_messages"] as! Int, reduction["prepared_messages"] as! Int)
        XCTAssertEqual(reduction["protected_groups_preserved"] as? Bool, true)
        XCTAssertLessThanOrEqual(reduction["final_tokens"] as! Int, reduction["input_allowance"] as! Int)
        // Mutating the persisted live roster precedes classifier admission.
        let selected = try start("orchestrator-agent", "ios-roster-\(rep)", "Calculate invoice arithmetic: 9 invoice items cost 4 USD each. Return JSON with exactly subtotal and currency.")
        let selectedEvents = try drain(selected)
        XCTAssertEqual(try result(selectedEvents), ["subtotal":36,"currency":"USD"] as NSDictionary)
        XCTAssertTrue(selectedEvents.contains { row in
            guard let event = row["event"] as? [String:Any], let data = event["data"] as? [String:Any] else { return false }
            return event["type"] as? String == "AgentSelected" && data["selected_agent_id"] as? String == agentID && data["roster_revision"] as? String != nil && data["selected_thread_id"] as? String != nil
        })
        let cancellation = try start(agentID, "ios-cancel-\(rep)", "List each integer from 1 through 10000 in JSON, computing its square.")
        XCTAssertEqual(try object(["op":"cancel", "run_id":cancellation])["acknowledged"] as? Bool, true)
        let cancellationEvents = try drain(cancellation, cancelled:true)
        let history = try call(["op":"history", "run_id":second]) as! [[String:Any]]
        XCTAssertEqual(history.compactMap { $0["event_id"] as? UInt64 }, secondEvents.compactMap { $0["id"] as? UInt64 })
        let close = try shutdown()
        try save(["schema_version":1,"phase":"seed","source":config["source"]!,"artifact":config["artifact"]!,"selection":config["selection"]!,"repetition":config["repetition"]!,"pid":ProcessInfo.processInfo.processIdentifier,"store":"store-\(rep)","agent":agent,"default_agent_challenge":defaultChallenge,"first":first,"second":second,"history":history,"first_events":firstEvents,"second_events":secondEvents,"skill_events":skillEvents,"context_run":contextRecord,"selection_events":selectedEvents,"cancellation_events":cancellationEvents,"shutdown":close], "seed-\(rep).json")
    }
    func testRestore() throws {
        let config = try configuration()
        let rep = String(describing: config["repetition"]!)
        let seed = try JSONSerialization.jsonObject(with: Data(contentsOf: documents.appendingPathComponent("seed-\(rep).json"))) as! [String:Any]
        XCTAssertNotEqual(seed["pid"] as? Int32, ProcessInfo.processInfo.processIdentifier)
        try open(config)
        defer { _ = try? call(["op":"shutdown"]) }
        let agent = seed["agent"] as! [String:Any]
        let restored = try object(["op":"agent","id":agent["id"]!])
        XCTAssertEqual(restored["prompt"] as? NSDictionary, agent["prompt"] as? NSDictionary)
        XCTAssertEqual(restored["metadata"] as? NSDictionary, agent["metadata"] as? NSDictionary)
        let global = try object(["op":"get_setting","key":"run_policy.global"])["value"] as! [String:Any]
        XCTAssertEqual((global["context_strategy"] as? [String:Any])?["max_messages"] as? Int, 16)
        let history = try call(["op":"history","run_id":seed["second"]!]) as! NSArray
        XCTAssertEqual(history, seed["history"] as! NSArray)
        let defaultNonce = UUID().uuidString.lowercased()
        XCTAssertNotEqual(defaultNonce, (seed["default_agent_challenge"] as? [String:Any])?["nonce"] as? String)
        let defaultOperand = Int.random(in: 10...999)
        let defaultMultiplier = Int.random(in: 2...9)
        let defaultIncrement = Int.random(in: 1...99)
        let defaultExpected: [String:Any] = ["nonce":defaultNonce,"result":defaultOperand * defaultMultiplier + defaultIncrement]
        let defaultRun = try start("default-agent", "ios-default-restore-\(rep)-\(defaultNonce)", "Compute (\(defaultOperand) multiplied by \(defaultMultiplier)) plus \(defaultIncrement). Return only a JSON object with exactly nonce and result; nonce is \(defaultNonce), result is the computed integer. Do not use tools.")
        let defaultEvents = try drain(defaultRun)
        XCTAssertEqual(try result(defaultEvents), defaultExpected as NSDictionary)
        let defaultChallenge: [String:Any] = ["nonce":defaultNonce,"operands":[defaultOperand,defaultMultiplier,defaultIncrement],"expected":defaultExpected,"run_id":defaultRun,"events":defaultEvents]
        try save(["schema_version":1,"phase":"restore","source":config["source"]!,"artifact":config["artifact"]!,"selection":config["selection"]!,"repetition":config["repetition"]!,"pid":ProcessInfo.processInfo.processIdentifier,"seed_pid":seed["pid"]!,"store":seed["store"]!,"agent_id":agent["id"]!,"run_id":seed["second"]!,"restored_history":history,"default_agent_challenge":defaultChallenge,"shutdown":try shutdown()],"restore-\(rep).json")
    }
}
