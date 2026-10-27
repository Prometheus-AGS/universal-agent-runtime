# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: features/a2ui-browser.feature.spec.js >> Interact with a real A2UI response in the browser >> A rendered calculator sends fresh input back to its agent
- Location: target/manual-a2ui-browser-generated/features/a2ui-browser.feature.spec.js:6:3

# Error details

```
AssertionError: Require genuine published A2UI artifact

0 !== 1

```

# Page snapshot

```yaml
- generic [ref=e3]:
  - banner [ref=e4]:
    - generic [ref=e5]: Universal Agent Runtime
    - navigation "Main navigation" [ref=e11]:
      - link "Chat" [ref=e12] [cursor=pointer]:
        - /url: /threads
      - link "Admin" [ref=e13] [cursor=pointer]:
        - /url: /admin
      - link "About" [ref=e14] [cursor=pointer]:
        - /url: /about
  - generic [ref=e16]:
    - complementary [ref=e17]:
      - generic [ref=e18]:
        - generic [ref=e19]: Threads
        - button "New thread" [ref=e20] [cursor=pointer]
      - textbox "Search threads" [ref=e26]:
        - /placeholder: Search threads…
      - generic [ref=e28]:
        - button "Please render the interactive parcel calculator. just now Default Assistant · I'll publish the parcel calculator surface now." [ref=e29] [cursor=pointer]:
          - generic [ref=e30]:
            - generic [ref=e31]:
              - paragraph [ref=e32]: Please render the interactive parcel calculator.
              - generic [ref=e33]: just now
            - paragraph [ref=e34]: Default Assistant · I'll publish the parcel calculator surface now.
        - button "Actions for Please render the interactive parcel calculator."
    - main [ref=e35]:
      - generic [ref=e36]:
        - button "Select agent" [ref=e37] [cursor=pointer]:
          - generic [ref=e38]:
            - generic [ref=e39]: Browser calculator 4b275ee4
            - generic [ref=e40]: liter/MiniMax-M3
        - button "Session configuration" [ref=e43] [cursor=pointer]
      - generic [ref=e45]:
        - generic [ref=e47]:
          - paragraph [ref=e51]: Please render the interactive parcel calculator.
          - generic [ref=e52]: You
        - generic [ref=e59]:
          - generic [ref=e60]:
            - generic [ref=e61]: Agent
            - generic [ref=e69]:
              - region "A2UI display artifact" [ref=e70]:
                - generic [ref=e71]:
                  - generic [ref=e75]: Artifact
                  - generic [ref=e76]: · a2ui
                - generic [ref=e77]:
                  - generic [ref=e78]:
                    - heading "Effective run policy" [level=3] [ref=e79]
                    - paragraph [ref=e80]: "[{\"version\":\"v0.9.1\",\"createSurface\":{\"surfaceId\":\"run-policy-eec4bf46-a399-4476-b98b-7414fb92ccaf\",\"catalogId\":\"urn:uar:a2ui:catalog:1\"}},{\"version\":\"v0.9.1\",\"updateComponents\":{\"surfaceId\":\"run-policy-eec4bf46-a399-4476-b98b-7414fb92ccaf\",\"components\":[{\"id\":\"title\",\"component\":\"Text\",\"text\":{\"path\":\"/title\"},\"variant\":\"h2\"},{\"id\":\"subtitle\",\"component\":\"Text\",\"text\":{\"path\":\"/subtitle\"},\"variant\":\"caption\"},{\"id\":\"overview-heading\",\"component\":\"Text\",\"text\":\"Execution\",\"variant\":\"h3\"},{\"id\":\"chat-mode\",\"component\":\"Text\",\"text\":{\"path\":\"/chatMode\"},\"variant\":\"body\"},{\"id\":\"model\",\"component\":\"Text\",\"text\":{\"path\":\"/model\"},\"variant\":\"body\"},{\"id\":\"agent\",\"component\":\"Text\",\"text\":{\"path\":\"/agent\"},\"variant\":\"body\"},{\"id\":\"overview-column\",\"component\":\"Column\",\"children\":[\"overview-heading\",\"chat-mode\",\"model\",\"agent\"]},{\"id\":\"overview-card\",\"component\":\"Card\",\"child\":\"overview-column\"},{\"id\":\"capabilities-heading\",\"component\":\"Text\",\"text\":\"Available capabilities\",\"variant\":\"h3\"},{\"id\":\"skills\",\"component\":\"Text\",\"text\":{\"path\":\"/skills\"},\"variant\":\"body\"},{\"id\":\"tools\",\"component\":\"Text\",\"text\":{\"path\":\"/tools\"},\"variant\":\"body\"},{\"id\":\"mcp-servers\",\"component\":\"Text\",\"text\":{\"path\":\"/mcpServers\"},\"variant\":\"body\"},{\"id\":\"knowledge-bases\",\"component\":\"Text\",\"text\":{\"path\":\"/knowledgeBases\"},\"variant\":\"body\"},{\"id\":\"capabilities-column\",\"component\":\"Column\",\"children\":[\"capabilities-heading\",\"skills\",\"tools\",\"mcp-servers\",\"knowledge-bases\"]},{\"id\":\"capabilities-card\",\"component\":\"Card\",\"child\":\"capabilities-column\"},{\"id\":\"runtime-heading\",\"component\":\"Text\",\"text\":\"Runtime controls\",\"variant\":\"h3\"},{\"id\":\"memory\",\"component\":\"Text\",\"text\":{\"path\":\"/memory\"},\"variant\":\"body\"},{\"id\":\"prompt-caching\",\"component\":\"Text\",\"text\":{\"path\":\"/promptCaching\"},\"variant\":\"body\"},{\"id\":\"context\",\"component\":\"Text\",\"text\":{\"path\":\"/context\"},\"variant\":\"body\"},{\"id\":\"approval\",\"component\":\"Text\",\"text\":{\"path\":\"/approval\"},\"variant\":\"body\"},{\"id\":\"runtime-column\",\"component\":\"Column\",\"children\":[\"runtime-heading\",\"memory\",\"prompt-caching\",\"context\",\"approval\"]},{\"id\":\"runtime-card\",\"component\":\"Card\",\"child\":\"runtime-column\"},{\"id\":\"warnings-heading\",\"component\":\"Text\",\"text\":{\"path\":\"/warningsHeading\"},\"variant\":\"h3\"},{\"id\":\"warnings\",\"component\":\"Text\",\"text\":{\"path\":\"/warnings\"},\"variant\":\"body\"},{\"id\":\"warnings-column\",\"component\":\"Column\",\"children\":[\"warnings-heading\",\"warnings\"]},{\"id\":\"warnings-card\",\"component\":\"Card\",\"child\":\"warnings-column\"},{\"id\":\"root\",\"component\":\"Column\",\"children\":[\"title\",\"subtitle\",\"overview-card\",\"capabilities-card\",\"runtime-card\",\"warnings-card\"]}]}},{\"version\":\"v0.9.1\",\"updateDataModel\":{\"surfaceId\":\"run-policy-eec4bf46-a399-4476-b98b-7414fb92ccaf\",\"path\":\"/\",\"value\":{\"title\":\"Effective run policy\",\"subtitle\":\"Resolved policy v1 · A2UI v0.9.1\",\"chatMode\":\"Chat mode · agent\",\"model\":\"Model · liter / MiniMax-M3\",\"agent\":\"Agent · disposable-a2ui-browser-f74a6eec-392b-445b-953f-f25b6d16a3fe\",\"skills\":\"Skills · auto · 210 available · agent scope\",\"tools\":\"Tools · selected · 1 available · agent scope\",\"mcpServers\":\"MCP servers · auto · 0 available · legacy scope\",\"knowledgeBases\":\"Knowledge bases · none · 0 available · agent scope\",\"memory\":\"Memory · on\",\"promptCaching\":\"Prompt caching · off\",\"context\":\"Context · sliding window\",\"approval\":\"Tool approval · auto\",\"warningsHeading\":\"Warnings · 0\",\"warnings\":\"No policy warnings.\"}}}]"
                  - status [ref=e81]
              - region "A2UI display artifact" [ref=e82]:
                - generic [ref=e83]:
                  - generic [ref=e87]: Artifact
                  - generic [ref=e88]: · json
                - generic [ref=e89]:
                  - generic [ref=e90]:
                    - heading "Turn manifest" [level=3] [ref=e91]
                    - paragraph [ref=e92]: "{\"schema_version\":1,\"manifest_hash\":\"f75e37df96b034e7eba6afa29c867b4c9f59f26df73bab808421973c28cd63a8\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-f74a6eec-392b-445b-953f-f25b6d16a3fe\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"9eb7f9c19e2fb8f69c85f56b82990aad4d5e0aa6782d066beb8592a21bf39cb0\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"e48a610b638bbcaa83edd221ed7b63975aaa8a43d29e1c52cfe85cf1dad1caea\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"},{\"id\":\"world_state.current_time\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"9b3f04777128f8e2d18f5eb1cd1a3215e3fa343481d507ff0c027e1ec6fa28cb\"},{\"id\":\"world_state.environment\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\"},{\"id\":\"world_state.permissions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\"},{\"id\":\"world_state.project_instructions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\"}],\"counts\":{\"total\":7,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1,\"world_state\":4},\"by_authority\":{\"host\":4,\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":55268,\"rendered_characters\":54764,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}"
                  - status [ref=e93]
              - region "A2UI display artifact" [ref=e94]:
                - generic [ref=e95]:
                  - generic [ref=e99]: Artifact
                  - generic [ref=e100]: · json
                - generic [ref=e101]:
                  - generic [ref=e102]:
                    - heading "resolved_step" [level=3] [ref=e103]
                    - paragraph [ref=e104]: "{\"source\":\"uar.turn\",\"payload\":{\"step\":1,\"model\":\"MiniMax-M3\",\"mcp_catalog\":[],\"manifest\":{\"schema_version\":1,\"manifest_hash\":\"57fe070344cfac6922c00fe24039357153d9741e4ab124b026ac39048f3e5608\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-f74a6eec-392b-445b-953f-f25b6d16a3fe\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"9eb7f9c19e2fb8f69c85f56b82990aad4d5e0aa6782d066beb8592a21bf39cb0\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"e48a610b638bbcaa83edd221ed7b63975aaa8a43d29e1c52cfe85cf1dad1caea\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"},{\"id\":\"world_state.current_time\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"9b3f04777128f8e2d18f5eb1cd1a3215e3fa343481d507ff0c027e1ec6fa28cb\"},{\"id\":\"world_state.environment\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\"},{\"id\":\"world_state.permissions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\"},{\"id\":\"world_state.project_instructions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\"}],\"counts\":{\"total\":7,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1,\"world_state\":4},\"by_authority\":{\"host\":4,\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":55037,\"rendered_characters\":54533,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}}}"
                  - status [ref=e105]
              - region "A2UI display artifact" [ref=e106]:
                - generic [ref=e107]:
                  - generic [ref=e111]: Artifact
                  - generic [ref=e112]: · json
                - generic [ref=e113]:
                  - generic [ref=e114]:
                    - heading "Provider attempt manifest" [level=3] [ref=e115]
                    - paragraph [ref=e116]: "{\"schema_version\":\"uar.attempt-manifest.v1\",\"destination_model\":\"MiniMax-M3\",\"budgeting\":{\"label\":\"legacy_unbudgeted\",\"contract_present\":false,\"fit_guarantee\":false},\"warning\":\"Provider dispatch is using the legacy unbudgeted path\"}"
                  - status [ref=e117]
              - region "A2UI display artifact" [ref=e118]:
                - generic [ref=e119]:
                  - generic [ref=e123]: Artifact
                  - generic [ref=e124]: · json
                - generic [ref=e125]:
                  - generic [ref=e126]:
                    - heading "budget_receipt" [level=3] [ref=e127]
                    - paragraph [ref=e128]: "{\"source\":\"uar.request\",\"payload\":{\"schema_version\":\"uar.request-budget.v1\",\"representation\":\"final_provider_request\",\"destination_model\":\"MiniMax-M3\",\"serialized_request\":true,\"count_quality\":\"approximate\",\"counting_revision\":\"approximate-cl100k-json-v1\",\"counted_input_tokens\":13792,\"serialized_bytes\":60402,\"serialized_sha256\":\"281956696eed86ce5fcb775a9b11268534400095bbc286cb4da895577c6b1646\",\"tools_included\":true,\"instructions_included\":true,\"fit_guarantee\":false,\"reserved_output_tokens\":null,\"output_ceiling_enforced\":true}}"
                  - status [ref=e129]
              - paragraph [ref=e131]: I'll publish the parcel calculator surface now.
              - button "a2ui_render complete" [ref=e133] [cursor=pointer]:
                - generic [ref=e134]: a2ui_render
                - generic [ref=e135]: complete
              - region "A2UI display artifact" [ref=e136]:
                - generic [ref=e137]:
                  - generic [ref=e141]: Artifact
                  - generic [ref=e142]: · json
                - generic [ref=e143]:
                  - generic [ref=e144]:
                    - heading "resolved_step" [level=3] [ref=e145]
                    - paragraph [ref=e146]: "{\"source\":\"uar.turn\",\"payload\":{\"step\":2,\"model\":\"MiniMax-M3\",\"mcp_catalog\":[],\"manifest\":{\"schema_version\":1,\"manifest_hash\":\"dbad9b2831d21a3be6f2b574330fc6e6ebdba9cad4ac16744837a9052a3f3247\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-f74a6eec-392b-445b-953f-f25b6d16a3fe\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"9eb7f9c19e2fb8f69c85f56b82990aad4d5e0aa6782d066beb8592a21bf39cb0\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"e48a610b638bbcaa83edd221ed7b63975aaa8a43d29e1c52cfe85cf1dad1caea\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"}],\"counts\":{\"total\":3,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1},\"by_authority\":{\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":56295,\"rendered_characters\":55791,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}}}"
                  - status [ref=e147]
              - region "A2UI display artifact" [ref=e148]:
                - generic [ref=e149]:
                  - generic [ref=e153]: Artifact
                  - generic [ref=e154]: · json
                - generic [ref=e155]:
                  - generic [ref=e156]:
                    - heading "Provider attempt manifest" [level=3] [ref=e157]
                    - paragraph [ref=e158]: "{\"schema_version\":\"uar.attempt-manifest.v1\",\"destination_model\":\"MiniMax-M3\",\"budgeting\":{\"label\":\"legacy_unbudgeted\",\"contract_present\":false,\"fit_guarantee\":false},\"warning\":\"Provider dispatch is using the legacy unbudgeted path\"}"
                  - status [ref=e159]
              - region "A2UI display artifact" [ref=e160]:
                - generic [ref=e161]:
                  - generic [ref=e165]: Artifact
                  - generic [ref=e166]: · json
                - generic [ref=e167]:
                  - generic [ref=e168]:
                    - heading "budget_receipt" [level=3] [ref=e169]
                    - paragraph [ref=e170]: "{\"source\":\"uar.request\",\"payload\":{\"schema_version\":\"uar.request-budget.v1\",\"representation\":\"final_provider_request\",\"destination_model\":\"MiniMax-M3\",\"serialized_request\":true,\"count_quality\":\"approximate\",\"counting_revision\":\"approximate-cl100k-json-v1\",\"counted_input_tokens\":14615,\"serialized_bytes\":63358,\"serialized_sha256\":\"ed02cad2d2af680ad00dc3c9842af55c6e025424b94603763939f2fc3367b27e\",\"tools_included\":true,\"instructions_included\":true,\"fit_guarantee\":false,\"reserved_output_tokens\":null,\"output_ceiling_enforced\":true}}"
                  - status [ref=e171]
              - generic [ref=e172]:
                - paragraph [ref=e173]:
                  - text: "<think>The issue: the \"action\" object's context field got mangled — it shows"
                  - code [ref=e174]: "\"context\": \"\""
                  - text: instead of
                  - code [ref=e175]: "\"context\": {}"
                  - text: . Actually, looking at my input, I did pass
                  - code [ref=e176]: "{\"context\":{}, \"event\":{...}}"
                  - text: . But the error shows
                  - code [ref=e177]: "\"action\":{\"context\":\"\",\"event\":{\"name\":\"calculate_parcel\"}}"
                  - text: . Hmm, it seems the tool serialized the empty object
                  - code [ref=e178]: "{}"
                  - text: as
                  - code [ref=e179]: "\"\""
                  - text: . Also
                  - code [ref=e180]: "{\"path\":\"/parcel\"}"
                  - text: might have been fine.
                - paragraph [ref=e181]:
                  - text: Wait — the error message shows
                  - code [ref=e182]: "context: \"\""
                  - text: . That's odd. Maybe the schema validation requires
                  - code [ref=e183]: context
                  - text: to be an object with additional properties, and empty object got transformed. Let me try re-sending with context as
                  - code [ref=e184]: "{}"
                  - text: again but perhaps the issue is that empty objects get stripped. Let me try making the context non-empty... but the fragment says context should be
                  - code [ref=e185]: "{}"
                  - text: .
                - paragraph [ref=e186]:
                  - text: Actually, maybe the issue is the "oneOf" failure — perhaps the whole message matched more than one schema, or the value
                  - code [ref=e187]: "{\"path\":\"/parcel\"}"
                  - text: "for TextField value is invalid. Let me reconsider: the schema says"
                  - code [ref=e188]: "value: {}"
                  - text: (any type).
                  - code [ref=e189]: "{\"path\":\"/parcel\"}"
                  - text: should be fine as a "path" binding object.
                - paragraph [ref=e190]:
                  - text: Hmm, the error specifically shows
                  - code [ref=e191]: "\"context\":\"\""
                  - text: . That suggests the empty object
                  - code [ref=e192]: "{}"
                  - text: was converted to empty string somewhere in serialization. Similarly, for
                  - code [ref=e193]: value
                  - text: ", it shows"
                  - code [ref=e194]: "{\"path\":\"/parcel\"}"
                  - text: preserved. So
                  - code [ref=e195]: "{}"
                  - text: →
                  - code [ref=e196]: "\"\""
                  - text: . That's a serialization quirk of empty objects.
                - paragraph [ref=e197]:
                  - text: "So I should avoid empty objects. But the spec says action event context: {}. Hmm. Maybe I could use"
                  - code [ref=e198]: "context: {\"parcel\": {\"path\": \"/parcel\"}}"
                  - text: "? But the requirement says action"
                  - code [ref=e199]: "{\"event\":{\"name\":\"calculate_parcel\",\"context\":{}}}"
                  - text: .
                - paragraph [ref=e200]: Let me try again — perhaps the conversion is non-deterministic or perhaps it only happened because... Actually, let me just try again and see. Alternatively the failure could be caused by something else entirely and the error message just renders differently.
                - paragraph [ref=e201]:
                  - text: "Actually let's think: the error is a JSON-schema oneOf failure on the whole message. The dump shown is the message as validated. If"
                  - code [ref=e202]: "context: \"\""
                  - text: were invalid against
                  - code [ref=e203]: "{\"type\":\"object\"}"
                  - text: ", the error would be about that property specifically, not oneOf at the top level for the message. The oneOf at the top level is on the union of message types: createSurface | updateComponents | updateDataModel. The message with updateComponents failed to match that schema."
                - paragraph [ref=e204]:
                  - text: Since
                  - code [ref=e205]: updateComponents
                  - text: "in the schema has properties: components (with contains requirement id const \"root\"), surfaceId, version. version enum [\"v0.9\",\"v0.9.1\"]. Components items schema: additionalProperties false, properties action, child, children, component, id, label, options, placeholder, text, value, variant; required [id, component]."
                - paragraph [ref=e206]:
                  - text: So
                  - code [ref=e207]: action
                  - text: must match
                  - code [ref=e208]: "{\"action\": {\"additionalProperties\": false, \"properties\": {\"event\": {...}}, \"required\": [\"event\"], \"type\": \"object\"}}"
                  - text: ". Note: the action schema has"
                  - code [ref=e209]: "additionalProperties: false"
                  - text: and only
                  - code [ref=e210]: event
                  - text: property! So
                  - code [ref=e211]: context
                  - text: at the action level would be disallowed. In my payload,
                  - code [ref=e212]: context
                  - text: is nested inside
                  - code [ref=e213]: event
                  - text: ", which has properties context and name — that's fine."
                - paragraph [ref=e214]:
                  - text: So
                  - code [ref=e215]: "action: {\"event\": {\"name\": \"calculate_parcel\", \"context\": {}}}"
                  - text: is correct. The error dump shows
                  - code [ref=e216]: "\"action\":{\"context\":\"\",\"event\":{\"name\":\"calculate_parcel\"}}"
                  - text: — the context moved OUT of event and became "" — that's the tool re-serializing/mangling. Hmm, that dump may just be a re-serialization of the received payload by the error reporter with key ordering changed (context before event alphabetically) and empty object shown as "".
                - paragraph [ref=e217]:
                  - text: Hmm, but why would
                  - code [ref=e218]: "{\"path\":\"/parcel\"}"
                  - text: be preserved and
                  - code [ref=e219]: "{}"
                  - text: "become \"\"? Possibly the error reporter uses a lossy format. Let's just try to work around: maybe use a context with a non-empty key? But requirement says context:{}."
                - paragraph [ref=e220]:
                  - text: "Let me first try re-sending the exact same message — maybe the mangling was in the display. But better: try sending with the value for TextField as"
                  - code [ref=e221]: "{\"path\": \"/parcel\"}"
                  - text: — that's fine.
                - paragraph [ref=e222]:
                  - text: Actually, wait. Maybe the real issue is the
                  - code [ref=e223]: contains
                  - text: "schema for components:"
                  - code [ref=e224]: "{\"contains\": {\"properties\": {\"id\": {\"const\": \"root\"}}, \"required\": [\"id\"]}, ...}"
                  - text: — our root has id "root" ✓.
                - paragraph [ref=e225]:
                  - text: "Let me consider: maybe the TextField"
                  - code [ref=e226]: value
                  - text: must be a string, not a path object. The basic catalog TextField's value might be typed. But schema here says
                  - code [ref=e227]: "value: {}"
                  - text: (unconstrained</think>
          - generic [ref=e229]:
            - button "Copy" [ref=e230] [cursor=pointer]
            - generic [ref=e232]:
              - 'generic "Answered by agent: Browser calculator 4b275ee4" [ref=e233]': Browser calculator 4b275ee4
              - 'generic "Model: MiniMax-M3" [ref=e237]': MiniMax-M3
              - 'generic "Tokens — in: 29183, out: 1539, total: 30722" [ref=e241]': "291831539"
        - generic [ref=e246]:
          - button "Scroll to bottom" [ref=e247] [cursor=pointer]
          - generic [ref=e249]:
            - button [ref=e250]
            - generic [ref=e251]:
              - textbox "Message input" [active] [ref=e252]:
                - /placeholder: Send a message to the agent…
              - generic [ref=e253]:
                - button "Attach file" [ref=e254] [cursor=pointer]
                - button "Disable memory for this message" [ref=e256] [cursor=pointer]:
                  - generic [ref=e257]: Memory ON – click to disable
                - button "Send message" [disabled]
          - generic [ref=e258]:
            - button "Toggle Knowledge" [ref=e259] [cursor=pointer]:
              - generic [ref=e260]: Knowledge
            - button "Toggle Tools" [ref=e261] [cursor=pointer]:
              - generic [ref=e262]: Tools
              - generic [ref=e263]: "1"
            - button "Toggle Skills" [ref=e264] [cursor=pointer]:
              - generic [ref=e265]: Skills
            - button "Enable Web Search" [ref=e266] [cursor=pointer]:
              - generic [ref=e267]: Web Search
            - button "Disable Memory" [ref=e268] [cursor=pointer]:
              - generic [ref=e269]: Memory
  - region "Notifications alt+T"
```

# Test source

```ts
  14  |     try { await use(state); } finally {
  15  |       if (state.created) {
  16  |         try {
  17  |           await api(state, 'DELETE', `/api/agents/${state.agentId}`, undefined, 204);
  18  |           state.evidence.disposable_agent_deleted = true;
  19  |         } catch (error) { cleanupError = error; state.evidence.cleanup_error = String(error); }
  20  |       }
  21  |       state.evidence.status = cleanupError ? 'failed' : testInfo.status;
  22  |       state.evidence.duration_ms = Math.round(performance.now() - state.started);
  23  |       const dir = process.env.UAR_BROWSER_EVIDENCE_DIR;
  24  |       await mkdir(dir, { recursive: true });
  25  |       await writeFile(resolve(dir, 'evidence.json'), JSON.stringify(state.evidence, null, 2) + '\n', { flag: 'wx' });
  26  |       if (cleanupError) throw cleanupError;
  27  |     }
  28  |   },
  29  | });
  30  | const { Given, When, Then } = createBdd(test);
  31  | 
  32  | async function api(state, method, path, payload, expected = 200) {
  33  |   const response = await fetch(new URL(path, process.env.UAR_BASE_URL), {
  34  |     method, redirect: 'error', signal: AbortSignal.timeout(15_000),
  35  |     headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID },
  36  |     body: payload === undefined ? undefined : JSON.stringify(payload),
  37  |   });
  38  |   if (method === 'POST' && path === '/api/agents' && response.status === 201) state.created = true;
  39  |   const text = await response.text();
  40  |   state.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text });
  41  |   assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  42  |   return text ? JSON.parse(text) : null;
  43  | }
  44  | 
  45  | async function stream(state, response) {
  46  |   const text = await response.text();
  47  |   const record = { path: new URL(response.url()).pathname, status: response.status(),
  48  |     request: response.request().postDataJSON(), response: text };
  49  |   state.evidence.streams.push(record);
  50  |   assert.equal(response.status(), 200);
  51  |   assert.match(response.headers()['content-type'], /^text\/event-stream\b/);
  52  |   const events = text.split(/\r?\n\r?\n/).flatMap(block => {
  53  |     const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
  54  |     const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
  55  |     if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
  56  |     return [JSON.parse(data)];
  57  |   });
  58  |   record.events = events;
  59  |   assert.ok(!events.some(event => ['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type)), 'Stream must succeed');
  60  |   const starts = events.filter(event => event.type === 'RUN_STARTED');
  61  |   const finishes = events.filter(event => event.type === 'RUN_FINISHED');
  62  |   assert.equal(starts.length, 1);
  63  |   assert.equal(finishes.length, 1);
  64  |   assert.equal(starts[0].runId, finishes[0].runId);
  65  |   return { runId: starts[0].runId, events,
  66  |     text: events.filter(event => event.type === 'TEXT_MESSAGE_CONTENT').map(event => event.delta).join('') };
  67  | }
  68  | 
  69  | Given('a disposable A2UI calculator agent is selected in the browser', async ({ page, scenario: s }) => {
  70  |   assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID);
  71  |   const url = new URL(process.env.UAR_BASE_URL);
  72  |   assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  73  |   s.agentId = `disposable-a2ui-browser-${randomUUID()}`;
  74  |   s.title = `Browser calculator ${randomUUID().slice(0, 8)}`;
  75  |   s.marker = `BROWSER-${randomUUID()}`;
  76  |   s.evidence.agent_id = s.agentId;
  77  |   s.evidence.model = process.env.UAR_MODEL;
  78  |   s.evidence.page_errors = [];
  79  |   page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  80  |   const agent = await api(s, 'GET', '/api/agents/default-agent');
  81  |   agent.id = s.agentId;
  82  |   agent.version = '1.0.0';
  83  |   agent.metadata = { ...agent.metadata, title: s.title, description: 'Disposable interactive parcel calculator.', tags: ['verification'] };
  84  |   agent.runtime.entry = 'default';
  85  |   agent.prompt = { system: 'On the initial request, call a2ui_render exactly once to publish a parcel calculator. '
  86  |     + 'Use A2UI version v0.9.1 with canonical createSurface, updateComponents and updateDataModel messages, '
  87  |     + 'catalog https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json. '
  88  |     + 'Create Column id root containing Text heading Parcel calculator, TextField labeled Parcel number with value {"path":"/parcel"}, '
  89  |     + 'and Button with a Text child reading Calculate parcel and action {"event":{"name":"calculate_parcel","context":{}}}. '
  90  |     + 'Initialize data model path / with value {"parcel":""}. Do not put UI JSON in prose. '
  91  |     + 'After publication give a brief summary. When a2ui.user_action arrives, read the fresh parcel value from interaction.a2uiClientDataModel under its surfaceId. '
  92  |     + `Do not render again. Return only JSON with marker "${s.marker}" and numeric result equal to the fresh parcel number plus 19.`, instructions: [] };
  93  |   const split = process.env.UAR_MODEL.indexOf('/');
  94  |   assert.ok(split > 0);
  95  |   agent.policy.provider = { default: { provider: process.env.UAR_MODEL.slice(0, split), model: process.env.UAR_MODEL.slice(split + 1) }, fallbacks: [] };
  96  |   agent.policy.tools.allow = ['a2ui_render'];
  97  |   agent.policy.skills.prefer = [];
  98  |   agent.memory.kb.enabled = false;
  99  |   const saved = await api(s, 'POST', '/api/agents', agent, 201);
  100 |   assert.equal(saved.id, s.agentId);
  101 |   await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  102 |   await page.getByRole('button', { name: 'New thread', exact: true }).click();
  103 |   await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  104 |   await page.getByPlaceholder('Search agents...').fill(s.title);
  105 |   await page.getByRole('option').filter({ hasText: s.title }).click();
  106 | });
  107 | 
  108 | When('I ask the agent to render its calculator', async ({ page, scenario: s }) => {
  109 |   const response = page.waitForResponse(r => r.request().method() === 'POST' && new URL(r.url()).pathname === '/api/chat/completion');
  110 |   await page.getByRole('textbox', { name: 'Message input', exact: true }).fill('Please render the interactive parcel calculator.');
  111 |   await page.getByRole('button', { name: 'Send message', exact: true }).click();
  112 |   s.initial = await stream(s, await response);
  113 |   const artifacts = s.initial.events.filter(event => event.type === 'CUSTOM' && event.name === 'uar.artifact.available' && event.value?.artifactType === 'a2ui');
> 114 |   assert.equal(artifacts.length, 1, 'Require genuine published A2UI artifact');
      |          ^ AssertionError: Require genuine published A2UI artifact
  115 |   assert.equal(artifacts[0].value.metadata?.sourceTool, 'a2ui_render');
  116 |   assert.equal(artifacts[0].value.metadata?.publication_status, 'published');
  117 |   s.evidence.source_run_id = s.initial.runId;
  118 | });
  119 | 
  120 | Then('the browser displays the generated input and action button', async ({ page, scenario: s }) => {
  121 |   s.surface = page.getByRole('region', { name: 'A2UI display artifact', exact: true });
  122 |   await expect(s.surface.locator('[data-a2ui-surface-state="ready"]')).toBeVisible();
  123 |   await expect(s.surface).toContainText('Parcel calculator');
  124 |   await expect(s.surface.getByRole('textbox', { name: 'Parcel number', exact: true })).toBeVisible();
  125 |   await expect(s.surface.getByRole('button', { name: 'Calculate parcel', exact: true })).toBeVisible();
  126 |   await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'rendered-surface.png'), fullPage: true });
  127 | });
  128 | 
  129 | When('I enter a fresh number and press the rendered calculate button', async ({ page, scenario: s }) => {
  130 |   s.number = randomInt(100, 900);
  131 |   s.evidence.fresh_input = s.number;
  132 |   await s.surface.getByRole('textbox', { name: 'Parcel number', exact: true }).fill(String(s.number));
  133 |   const action = page.waitForResponse(r => r.request().method() === 'POST' && /\/api\/uar\/runs\/[^/]+\/a2ui\/actions$/.test(new URL(r.url()).pathname));
  134 |   const continuation = page.waitForResponse(r => r.request().method() === 'GET' && /\/api\/uar\/runs\/[^/]+\/stream$/.test(new URL(r.url()).pathname));
  135 |   await s.surface.getByRole('button', { name: 'Calculate parcel', exact: true }).click();
  136 |   const response = await action;
  137 |   const body = response.request().postDataJSON();
  138 |   const ack = await response.json();
  139 |   s.evidence.action = { path: new URL(response.url()).pathname, request: body, status: response.status(), response: ack };
  140 |   assert.equal(response.status(), 202);
  141 |   assert.equal(body.name, 'calculate_parcel');
  142 |   assert.equal(body.a2uiClientDataModel?.[body.surfaceId]?.parcel, String(s.number));
  143 |   assert.equal(ack.status, 'continued');
  144 |   assert.equal(ack.run_id, s.initial.runId);
  145 |   assert.ok(ack.continuation_run_id && ack.continuation_run_id !== ack.run_id);
  146 |   s.continuation = await stream(s, await continuation);
  147 |   assert.equal(s.continuation.runId, ack.continuation_run_id);
  148 | });
  149 | 
  150 | Then('the agent answers using that browser input and its saved instructions', async ({ page, scenario: s }) => {
  151 |   const final = s.continuation.text.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  152 |   let answer;
  153 |   try { answer = JSON.parse(final); } catch {
  154 |     const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
  155 |     assert.equal(fences.length, 1);
  156 |     answer = JSON.parse(fences[0][1]);
  157 |   }
  158 |   assert.deepEqual(answer, { marker: s.marker, result: s.number + 19 });
  159 |   const last = page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]').last();
  160 |   await expect(last).toContainText(s.marker);
  161 |   await expect(last).toContainText(String(s.number + 19));
  162 |   s.evidence.answer = answer;
  163 |   s.evidence.continuation_run_id = s.continuation.runId;
  164 |   s.evidence.result = 'VERIFIED';
  165 |   await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'continued-answer.png'), fullPage: true });
  166 | });
  167 | 
```