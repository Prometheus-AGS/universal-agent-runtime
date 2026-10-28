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
        - button "Please render the interactive parcel calculator. just now Default Assistant · <think>Need to render exactly once using the templ..." [ref=e29] [cursor=pointer]:
          - generic [ref=e30]:
            - generic [ref=e31]:
              - paragraph [ref=e32]: Please render the interactive parcel calculator.
              - generic [ref=e33]: just now
            - paragraph [ref=e34]: Default Assistant · <think>Need to render exactly once using the templ...
        - button "Actions for Please render the interactive parcel calculator."
    - main [ref=e35]:
      - generic [ref=e36]:
        - button "Select agent" [ref=e37] [cursor=pointer]:
          - generic [ref=e38]:
            - generic [ref=e39]: Browser calculator 6f9c1eff
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
                    - paragraph [ref=e80]: "[{\"version\":\"v0.9.1\",\"createSurface\":{\"surfaceId\":\"run-policy-4a351495-be50-4936-8fbf-9a218e74a128\",\"catalogId\":\"urn:uar:a2ui:catalog:1\"}},{\"version\":\"v0.9.1\",\"updateComponents\":{\"surfaceId\":\"run-policy-4a351495-be50-4936-8fbf-9a218e74a128\",\"components\":[{\"id\":\"title\",\"component\":\"Text\",\"text\":{\"path\":\"/title\"},\"variant\":\"h2\"},{\"id\":\"subtitle\",\"component\":\"Text\",\"text\":{\"path\":\"/subtitle\"},\"variant\":\"caption\"},{\"id\":\"overview-heading\",\"component\":\"Text\",\"text\":\"Execution\",\"variant\":\"h3\"},{\"id\":\"chat-mode\",\"component\":\"Text\",\"text\":{\"path\":\"/chatMode\"},\"variant\":\"body\"},{\"id\":\"model\",\"component\":\"Text\",\"text\":{\"path\":\"/model\"},\"variant\":\"body\"},{\"id\":\"agent\",\"component\":\"Text\",\"text\":{\"path\":\"/agent\"},\"variant\":\"body\"},{\"id\":\"overview-column\",\"component\":\"Column\",\"children\":[\"overview-heading\",\"chat-mode\",\"model\",\"agent\"]},{\"id\":\"overview-card\",\"component\":\"Card\",\"child\":\"overview-column\"},{\"id\":\"capabilities-heading\",\"component\":\"Text\",\"text\":\"Available capabilities\",\"variant\":\"h3\"},{\"id\":\"skills\",\"component\":\"Text\",\"text\":{\"path\":\"/skills\"},\"variant\":\"body\"},{\"id\":\"tools\",\"component\":\"Text\",\"text\":{\"path\":\"/tools\"},\"variant\":\"body\"},{\"id\":\"mcp-servers\",\"component\":\"Text\",\"text\":{\"path\":\"/mcpServers\"},\"variant\":\"body\"},{\"id\":\"knowledge-bases\",\"component\":\"Text\",\"text\":{\"path\":\"/knowledgeBases\"},\"variant\":\"body\"},{\"id\":\"capabilities-column\",\"component\":\"Column\",\"children\":[\"capabilities-heading\",\"skills\",\"tools\",\"mcp-servers\",\"knowledge-bases\"]},{\"id\":\"capabilities-card\",\"component\":\"Card\",\"child\":\"capabilities-column\"},{\"id\":\"runtime-heading\",\"component\":\"Text\",\"text\":\"Runtime controls\",\"variant\":\"h3\"},{\"id\":\"memory\",\"component\":\"Text\",\"text\":{\"path\":\"/memory\"},\"variant\":\"body\"},{\"id\":\"prompt-caching\",\"component\":\"Text\",\"text\":{\"path\":\"/promptCaching\"},\"variant\":\"body\"},{\"id\":\"context\",\"component\":\"Text\",\"text\":{\"path\":\"/context\"},\"variant\":\"body\"},{\"id\":\"approval\",\"component\":\"Text\",\"text\":{\"path\":\"/approval\"},\"variant\":\"body\"},{\"id\":\"runtime-column\",\"component\":\"Column\",\"children\":[\"runtime-heading\",\"memory\",\"prompt-caching\",\"context\",\"approval\"]},{\"id\":\"runtime-card\",\"component\":\"Card\",\"child\":\"runtime-column\"},{\"id\":\"warnings-heading\",\"component\":\"Text\",\"text\":{\"path\":\"/warningsHeading\"},\"variant\":\"h3\"},{\"id\":\"warnings\",\"component\":\"Text\",\"text\":{\"path\":\"/warnings\"},\"variant\":\"body\"},{\"id\":\"warnings-column\",\"component\":\"Column\",\"children\":[\"warnings-heading\",\"warnings\"]},{\"id\":\"warnings-card\",\"component\":\"Card\",\"child\":\"warnings-column\"},{\"id\":\"root\",\"component\":\"Column\",\"children\":[\"title\",\"subtitle\",\"overview-card\",\"capabilities-card\",\"runtime-card\",\"warnings-card\"]}]}},{\"version\":\"v0.9.1\",\"updateDataModel\":{\"surfaceId\":\"run-policy-4a351495-be50-4936-8fbf-9a218e74a128\",\"path\":\"/\",\"value\":{\"title\":\"Effective run policy\",\"subtitle\":\"Resolved policy v1 · A2UI v0.9.1\",\"chatMode\":\"Chat mode · agent\",\"model\":\"Model · liter / MiniMax-M3\",\"agent\":\"Agent · disposable-a2ui-browser-668951b5-7701-471b-bde4-053c5240eeaf\",\"skills\":\"Skills · auto · 210 available · agent scope\",\"tools\":\"Tools · selected · 1 available · agent scope\",\"mcpServers\":\"MCP servers · auto · 0 available · legacy scope\",\"knowledgeBases\":\"Knowledge bases · none · 0 available · agent scope\",\"memory\":\"Memory · on\",\"promptCaching\":\"Prompt caching · off\",\"context\":\"Context · sliding window\",\"approval\":\"Tool approval · auto\",\"warningsHeading\":\"Warnings · 0\",\"warnings\":\"No policy warnings.\"}}}]"
                  - status [ref=e81]
              - region "A2UI display artifact" [ref=e82]:
                - generic [ref=e83]:
                  - generic [ref=e87]: Artifact
                  - generic [ref=e88]: · json
                - generic [ref=e89]:
                  - generic [ref=e90]:
                    - heading "Turn manifest" [level=3] [ref=e91]
                    - paragraph [ref=e92]: "{\"schema_version\":1,\"manifest_hash\":\"6cc3ae307075f6360b6bbf37845a72792d3d0aa71dbc7751332dd5903c2893f9\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-668951b5-7701-471b-bde4-053c5240eeaf\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"4f5108126c02294d4fea07ffcf5131aac825490d15f57f6bd36a1c24b7fa166c\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"c0376c8ad68c2f26b92aa638bf6e77ae59f96a26054338c5ea8fd0f9d3464c6f\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"},{\"id\":\"world_state.current_time\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"315c3b572b08ea80b86c1df5106879eadb4127193a4c3dbe95426792defd6d0c\"},{\"id\":\"world_state.environment\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\"},{\"id\":\"world_state.permissions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\"},{\"id\":\"world_state.project_instructions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\"}],\"counts\":{\"total\":7,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1,\"world_state\":4},\"by_authority\":{\"host\":4,\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":55622,\"rendered_characters\":55118,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}"
                  - status [ref=e93]
              - region "A2UI display artifact" [ref=e94]:
                - generic [ref=e95]:
                  - generic [ref=e99]: Artifact
                  - generic [ref=e100]: · json
                - generic [ref=e101]:
                  - generic [ref=e102]:
                    - heading "resolved_step" [level=3] [ref=e103]
                    - paragraph [ref=e104]: "{\"source\":\"uar.turn\",\"payload\":{\"step\":1,\"model\":\"MiniMax-M3\",\"mcp_catalog\":[],\"manifest\":{\"schema_version\":1,\"manifest_hash\":\"889e12ee7ec2b349e65e2c11d7c979532eecc7968b56e834cf589a4419206a15\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-668951b5-7701-471b-bde4-053c5240eeaf\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"4f5108126c02294d4fea07ffcf5131aac825490d15f57f6bd36a1c24b7fa166c\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"c0376c8ad68c2f26b92aa638bf6e77ae59f96a26054338c5ea8fd0f9d3464c6f\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"},{\"id\":\"world_state.current_time\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"315c3b572b08ea80b86c1df5106879eadb4127193a4c3dbe95426792defd6d0c\"},{\"id\":\"world_state.environment\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"60dd57dd0b5613817cff0f9517e6e5ec987d6ee6cd6e0bc93bbfbd5d65d5e859\"},{\"id\":\"world_state.permissions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"24ebd0230f6d7165bb84398bb8f16f1b21ca7592192806ef347039d0e2a8bd3d\"},{\"id\":\"world_state.project_instructions\",\"section\":\"world_state\",\"source\":\"host.world_state\",\"authority\":\"host\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"ae02a71f9847b4174f62323d69e05ded8921eb7d1ee73ca6c34f99f55ed16dad\"}],\"counts\":{\"total\":7,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1,\"world_state\":4},\"by_authority\":{\"host\":4,\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":55391,\"rendered_characters\":54887,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}}}"
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
                    - paragraph [ref=e128]: "{\"source\":\"uar.request\",\"payload\":{\"schema_version\":\"uar.request-budget.v1\",\"representation\":\"final_provider_request\",\"destination_model\":\"MiniMax-M3\",\"serialized_request\":true,\"count_quality\":\"approximate\",\"counting_revision\":\"approximate-cl100k-json-v1\",\"counted_input_tokens\":13899,\"serialized_bytes\":60866,\"serialized_sha256\":\"bdd5ee62f86665cf28141e9f6c5c0dc99c58a59ebe1226f73feff869865ea540\",\"tools_included\":true,\"instructions_included\":true,\"fit_guarantee\":false,\"reserved_output_tokens\":null,\"output_ceiling_enforced\":true}}"
                  - status [ref=e129]
              - generic [ref=e130]:
                - paragraph [ref=e131]: <think>Need to render exactly once using the template verbatim. On the initial request call, a2ui_render is called exactly once with this valid template verbatim. The template is provided in the uar-fragment agent_identity, and should not be altered.
                - paragraph [ref=e132]: No other tool should be used or the template printed as prose. After publication, say only "Calculator ready."
                - paragraph [ref=e133]: The tool schema expects the messages array with version and createSurface/updateComponents/updateDataModel entries. The template provided is a JSON string of the full messages array, so that will be the argument for messages.
                - paragraph [ref=e134]: "Call the tool exactly once with the template unchanged. The template contains \"path\":\"/parcel\" in TextField value, and updateDataModel path \"/\" value {\"parcel\":\"\"}. The schema allows value to be any type."
                - paragraph [ref=e135]: After tool returns, respond with only "Calculator ready."</think>
              - button "a2ui_render complete" [ref=e137] [cursor=pointer]:
                - generic [ref=e138]: a2ui_render
                - generic [ref=e139]: complete
              - region "A2UI display artifact" [ref=e140]:
                - generic [ref=e141]:
                  - generic [ref=e145]: Artifact
                  - generic [ref=e146]: · json
                - generic [ref=e147]:
                  - generic [ref=e148]:
                    - heading "resolved_step" [level=3] [ref=e149]
                    - paragraph [ref=e150]: "{\"source\":\"uar.turn\",\"payload\":{\"step\":2,\"model\":\"MiniMax-M3\",\"mcp_catalog\":[],\"manifest\":{\"schema_version\":1,\"manifest_hash\":\"2a9130a6e9226367c92e9f64551ce4a2ef69e3eda4907cb04e97bfb88d2c7004\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-668951b5-7701-471b-bde4-053c5240eeaf\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"4f5108126c02294d4fea07ffcf5131aac825490d15f57f6bd36a1c24b7fa166c\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"c0376c8ad68c2f26b92aa638bf6e77ae59f96a26054338c5ea8fd0f9d3464c6f\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"}],\"counts\":{\"total\":3,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1},\"by_authority\":{\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":57371,\"rendered_characters\":56867,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}}}"
                  - status [ref=e151]
              - region "A2UI display artifact" [ref=e152]:
                - generic [ref=e153]:
                  - generic [ref=e157]: Artifact
                  - generic [ref=e158]: · json
                - generic [ref=e159]:
                  - generic [ref=e160]:
                    - heading "Provider attempt manifest" [level=3] [ref=e161]
                    - paragraph [ref=e162]: "{\"schema_version\":\"uar.attempt-manifest.v1\",\"destination_model\":\"MiniMax-M3\",\"budgeting\":{\"label\":\"legacy_unbudgeted\",\"contract_present\":false,\"fit_guarantee\":false},\"warning\":\"Provider dispatch is using the legacy unbudgeted path\"}"
                  - status [ref=e163]
              - region "A2UI display artifact" [ref=e164]:
                - generic [ref=e165]:
                  - generic [ref=e169]: Artifact
                  - generic [ref=e170]: · json
                - generic [ref=e171]:
                  - generic [ref=e172]:
                    - heading "budget_receipt" [level=3] [ref=e173]
                    - paragraph [ref=e174]: "{\"source\":\"uar.request\",\"payload\":{\"schema_version\":\"uar.request-budget.v1\",\"representation\":\"final_provider_request\",\"destination_model\":\"MiniMax-M3\",\"serialized_request\":true,\"count_quality\":\"approximate\",\"counting_revision\":\"approximate-cl100k-json-v1\",\"counted_input_tokens\":14868,\"serialized_bytes\":64461,\"serialized_sha256\":\"afb2af2c7ece957fecf9975dafcf20cf62f04e5eb4c56a08411376984d81b567\",\"tools_included\":true,\"instructions_included\":true,\"fit_guarantee\":false,\"reserved_output_tokens\":null,\"output_ceiling_enforced\":true}}"
                  - status [ref=e175]
              - generic [ref=e176]:
                - paragraph [ref=e177]:
                  - text: <think>The tool call failed because I wrapped the components array incorrectly — I passed
                  - code [ref=e178]: components
                  - text: as an array containing an array. The template says components is an array of component objects. But the instruction says call a2ui_render exactly once with the valid template verbatim, and do not alter the template, use another tool or print it as prose.
                - paragraph [ref=e179]: I already called once (it errored). Hmm. The instruction says "On the initial request call a2ui_render exactly once with this valid template verbatim". I made an error in serialization — I nested arrays incorrectly due to my own construction. Now the call failed.
                - paragraph [ref=e180]: Should I retry? The instruction says call exactly once. The call failed though, so publication hasn't happened. "After publication say only Calculator ready." Since it failed, there's no publication. The correct action is to retry with the correct arguments matching the schema — the template intent is a flat array of components. The error message shows the schema requires components as a flat array.
                - paragraph [ref=e181]: "However, the rule \"call a2ui_render exactly once\" — a failed call arguably doesn't count as a successful publication. Retrying with the verbatim template (flat components array) seems reasonable and necessary. But wait — the strict instruction is explicit: \"call a2ui_render exactly once with this valid template verbatim... Do not alter the template, use another tool or print it as prose.\""
                - paragraph [ref=e182]: "The intent: render the calculator. A malformed call that errored doesn't render. Retrying with the proper flat structure is completing the intent. I think retrying is correct — the important thing is the final published surface matches the template exactly. Risk: a strict checker counting tool calls. But the alternative (leaving nothing rendered and saying \"Calculator ready\") would be false."
                - paragraph [ref=e183]: "Let me retry once with the exact template structure: messages array with three message objects, components a flat array of 5 component objects."
                - paragraph [ref=e184]: "Actually, careful: the enforcement might block a second call. Let's attempt it — if it works, say \"Calculator ready.\" Only call again if first failed, which it did."
              - button "a2ui_render complete" [ref=e186] [cursor=pointer]:
                - generic [ref=e187]: a2ui_render
                - generic [ref=e188]: complete
              - region "A2UI display artifact" [ref=e189]:
                - generic [ref=e190]:
                  - generic [ref=e194]: Artifact
                  - generic [ref=e195]: · json
                - generic [ref=e196]:
                  - generic [ref=e197]:
                    - heading "resolved_step" [level=3] [ref=e198]
                    - paragraph [ref=e199]: "{\"source\":\"uar.turn\",\"payload\":{\"step\":3,\"model\":\"MiniMax-M3\",\"mcp_catalog\":[],\"manifest\":{\"schema_version\":1,\"manifest_hash\":\"18bac825d90bce37063cb0c0605d3485e0149a4727f31c03f64775e77ab8febd\",\"fragments\":[{\"id\":\"agent.identity\",\"section\":\"agent_identity\",\"source\":\"artifact:disposable-a2ui-browser-668951b5-7701-471b-bde4-053c5240eeaf\",\"authority\":\"system\",\"role\":\"system\",\"retention\":\"session\",\"content_hash\":\"4f5108126c02294d4fea07ffcf5131aac825490d15f57f6bd36a1c24b7fa166c\"},{\"id\":\"policy.effective\",\"section\":\"enforced_policy\",\"source\":\"effective_run_policy\",\"authority\":\"policy\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"c0376c8ad68c2f26b92aa638bf6e77ae59f96a26054338c5ea8fd0f9d3464c6f\"},{\"id\":\"skills.catalog\",\"section\":\"skill_catalog\",\"source\":\"eligible_skill_registry\",\"authority\":\"skill\",\"role\":\"system\",\"retention\":\"turn\",\"content_hash\":\"d77ed6dbeef567fdfafab070e307d297b37ad4baaa1ddd06bcacff342b99ce29\"}],\"counts\":{\"total\":3,\"by_section\":{\"agent_identity\":1,\"enforced_policy\":1,\"skill_catalog\":1},\"by_authority\":{\"policy\":1,\"skill\":1,\"system\":1}},\"budgets\":{\"rendered_bytes\":60627,\"rendered_characters\":60109,\"context_window_tokens\":1048576},\"selected_skills\":[],\"selected_tools\":[\"a2ui_render\",\"activate_skill\"],\"warnings\":[]}}}"
                  - status [ref=e200]
              - region "A2UI display artifact" [ref=e201]:
                - generic [ref=e202]:
                  - generic [ref=e206]: Artifact
                  - generic [ref=e207]: · json
                - generic [ref=e208]:
                  - generic [ref=e209]:
                    - heading "Provider attempt manifest" [level=3] [ref=e210]
                    - paragraph [ref=e211]: "{\"schema_version\":\"uar.attempt-manifest.v1\",\"destination_model\":\"MiniMax-M3\",\"budgeting\":{\"label\":\"legacy_unbudgeted\",\"contract_present\":false,\"fit_guarantee\":false},\"warning\":\"Provider dispatch is using the legacy unbudgeted path\"}"
                  - status [ref=e212]
              - region "A2UI display artifact" [ref=e213]:
                - generic [ref=e214]:
                  - generic [ref=e218]: Artifact
                  - generic [ref=e219]: · json
                - generic [ref=e220]:
                  - generic [ref=e221]:
                    - heading "budget_receipt" [level=3] [ref=e222]
                    - paragraph [ref=e223]: "{\"source\":\"uar.request\",\"payload\":{\"schema_version\":\"uar.request-budget.v1\",\"representation\":\"final_provider_request\",\"destination_model\":\"MiniMax-M3\",\"serialized_request\":true,\"count_quality\":\"approximate\",\"counting_revision\":\"approximate-cl100k-json-v1\",\"counted_input_tokens\":16076,\"serialized_bytes\":69332,\"serialized_sha256\":\"e977de5516aae8062fd89e346630b2b61d1379409b18941db6f8f02c3303e933\",\"tools_included\":true,\"instructions_included\":true,\"fit_guarantee\":false,\"reserved_output_tokens\":null,\"output_ceiling_enforced\":true}}"
                  - status [ref=e224]
              - generic [ref=e225]:
                - paragraph [ref=e226]:
                  - text: "<think>The serialization keeps producing nested arrays. This appears to be my output formatting issue where an array of items is being wrapped into {\"item\":[...]} — meaning my JSON emission is being interpreted as a list wrapper. It seems the tool-call emitter is turning arrays into a single-element array containing the array? Actually the error shows"
                  - code [ref=e227]: "\"components\":{\"item\":[...]}"
                  - text: — i.e., the array got converted into an object with "item" key, and inside "item" the root component has
                  - code [ref=e228]: "\"children\":{\"item\":[...]}"
                  - text: .
                - paragraph [ref=e229]: "So something in my emission is adding {\"item\": ...} wrappers. This looks like the harness converts my JSON arrays of objects into a schema-normalized form. Hmm."
                - paragraph [ref=e230]: Perhaps I should emit the components differently — maybe the issue is that the tool-call format wraps arrays. Let me try emitting the whole thing but with components as a JSON string? No.
                - paragraph [ref=e231]:
                  - text: "Alternative: maybe I should use the exact original template string as a single string argument? The tool's"
                  - code [ref=e232]: messages
                  - text: parameter expects an array of message objects. Hmm.
                - paragraph [ref=e233]: "Let me think about what's happening: The tool wrapper seems to take my JSON and, for array-typed fields, wrap them as {\"item\": [...]}. That's a known pattern for XML-ish serialization where repeated elements need \"item\" wrappers. So perhaps the accepted format for the host is to write arrays directly, but the harness is transforming them wrongly."
                - paragraph [ref=e234]:
                  - text: "Actually, the error message echoes my JSON:"
                  - code [ref=e235]: "\"components\":{\"item\":[{...}]}"
                  - text: . So the harness transformed my
                  - code [ref=e236]: "components: [...]"
                  - text: into
                  - code [ref=e237]: "{\"item\": [...]}"
                  - text: . That transformation is applied by the tool-call generator. To avoid it, maybe I should pass the components array such that... hmm, I can't control the transformation except by using a different representation.
                - paragraph [ref=e238]: "Possibility: the transformation happens because I'm emitting JSON in a particular way. Let me try emitting the arguments as a JSON string (the tool parameter typed as object might accept a string?). Unlikely."
                - paragraph [ref=e239]: Let me try once more but write the JSON compactly without newlines/indentation. Perhaps the pretty-printed form triggers the transformation. Worth a try.
                - paragraph [ref=e240]:
                  - text: Actually — note the nested
                  - code [ref=e241]: "\"children\":{\"item\":[\"heading\",\"parcel\",\"calculate\"]}"
                  - text: "also got wrapped. So all arrays get wrapped into {\"item\": [...]}. This is systematic. Compact formatting might still produce the same. But it's worth trying."
                - paragraph [ref=e242]: "Another thought: The tool call format may accept arguments where I write the JSON with arrays, and the harness's conversion is a bug in this environment; but perhaps repeated attempts with compact JSON succeed. Let's try compact."
                - paragraph [ref=e243]: I'll attempt a third call with compact single-line JSON.
                - paragraph [ref=e244]: If it fails again, I should report the failure honestly. But the policy demands only "Calculator ready" after publication. Let's try compact first.
          - generic [ref=e246]:
            - button "Copy" [ref=e247] [cursor=pointer]
            - generic [ref=e249]:
              - 'generic "Answered by agent: Browser calculator 6f9c1eff" [ref=e250]': Browser calculator 6f9c1eff
              - 'generic "Model: MiniMax-M3" [ref=e254]': MiniMax-M3
              - 'generic "Tokens — in: 46023, out: 2554, total: 48577" [ref=e258]': "460232554"
        - generic [ref=e263]:
          - button "Scroll to bottom" [ref=e264] [cursor=pointer]
          - generic [ref=e266]:
            - button [ref=e267]
            - generic [ref=e268]:
              - textbox "Message input" [active] [ref=e269]:
                - /placeholder: Send a message to the agent…
              - generic [ref=e270]:
                - button "Attach file" [ref=e271] [cursor=pointer]
                - button "Disable memory for this message" [ref=e273] [cursor=pointer]:
                  - generic [ref=e274]: Memory ON – click to disable
                - button "Send message" [disabled]
          - generic [ref=e275]:
            - button "Toggle Knowledge" [ref=e276] [cursor=pointer]:
              - generic [ref=e277]: Knowledge
            - button "Toggle Tools" [ref=e278] [cursor=pointer]:
              - generic [ref=e279]: Tools
              - generic [ref=e280]: "1"
            - button "Toggle Skills" [ref=e281] [cursor=pointer]:
              - generic [ref=e282]: Skills
            - button "Enable Web Search" [ref=e283] [cursor=pointer]:
              - generic [ref=e284]: Web Search
            - button "Disable Memory" [ref=e285] [cursor=pointer]:
              - generic [ref=e286]: Memory
  - region "Notifications alt+T"
```

# Test source

```ts
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
  45  | function observeResponse(page, predicate) {
  46  |   // Attach rejection handling immediately so an earlier UI failure stays primary.
  47  |   const pending = page.waitForResponse(predicate).then(response => ({ response }), error => ({ error }));
  48  |   return async () => {
  49  |     const result = await pending;
  50  |     if (result.error) throw result.error;
  51  |     return result.response;
  52  |   };
  53  | }
  54  | 
  55  | async function stream(state, response) {
  56  |   const text = await response.text();
  57  |   const record = { path: new URL(response.url()).pathname, status: response.status(),
  58  |     request: response.request().postDataJSON(), response: text };
  59  |   state.evidence.streams.push(record);
  60  |   assert.equal(response.status(), 200);
  61  |   assert.match(response.headers()['content-type'], /^text\/event-stream\b/);
  62  |   const events = text.split(/\r?\n\r?\n/).flatMap(block => {
  63  |     const name = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
  64  |     const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
  65  |     if (!data || data === '[DONE]' || name?.startsWith('runtime.')) return [];
  66  |     return [JSON.parse(data)];
  67  |   });
  68  |   record.events = events;
  69  |   assert.ok(!events.some(event => ['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type)), 'Stream must succeed');
  70  |   const starts = events.filter(event => event.type === 'RUN_STARTED');
  71  |   const finishes = events.filter(event => event.type === 'RUN_FINISHED');
  72  |   assert.equal(starts.length, 1);
  73  |   assert.equal(finishes.length, 1);
  74  |   assert.equal(starts[0].runId, finishes[0].runId);
  75  |   return { runId: starts[0].runId, events,
  76  |     text: events.filter(event => event.type === 'TEXT_MESSAGE_CONTENT').map(event => event.delta).join('') };
  77  | }
  78  | 
  79  | Given('a disposable A2UI calculator agent is selected in the browser', async ({ page, scenario: s }) => {
  80  |   assert.ok(process.env.UAR_MODEL && process.env.UAR_WORKSPACE_ID);
  81  |   const url = new URL(process.env.UAR_BASE_URL);
  82  |   assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  83  |   s.agentId = `disposable-a2ui-browser-${randomUUID()}`;
  84  |   s.title = `Browser calculator ${randomUUID().slice(0, 8)}`;
  85  |   s.marker = `BROWSER-${randomUUID()}`;
  86  |   s.evidence.agent_id = s.agentId;
  87  |   s.evidence.model = process.env.UAR_MODEL;
  88  |   s.evidence.page_errors = [];
  89  |   page.on('pageerror', error => s.evidence.page_errors.push(String(error)));
  90  |   const agent = await api(s, 'GET', '/api/agents/default-agent');
  91  |   agent.id = s.agentId;
  92  |   agent.version = '1.0.0';
  93  |   agent.metadata = { ...agent.metadata, title: s.title, description: 'Disposable interactive parcel calculator.', tags: ['verification'] };
  94  |   agent.runtime.entry = 'default';
  95  |   const template = {"messages":[{"version":"v0.9.1","createSurface":{"surfaceId":"parcel-calculator","catalogId":"https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json"}},{"version":"v0.9.1","updateComponents":{"surfaceId":"parcel-calculator","components":[{"id":"root","component":"Column","children":["heading","parcel","calculate"]},{"id":"heading","component":"Text","text":"Parcel calculator"},{"id":"parcel","component":"TextField","label":"Parcel number","value":{"path":"/parcel"}},{"id":"calculate","component":"Button","child":"button-label","action":{"event":{"name":"calculate_parcel"}}},{"id":"button-label","component":"Text","text":"Calculate parcel"}]}},{"version":"v0.9.1","updateDataModel":{"surfaceId":"parcel-calculator","path":"/","value":{"parcel":""}}}]};
  96  |   agent.prompt = { system: 'On the initial request call a2ui_render exactly once with this valid template verbatim: '
  97  |     + JSON.stringify(template) + '. Do not alter the template, use another tool or print it as prose. '
  98  |     + 'After publication say only Calculator ready. '
  99  |     + 'On an a2ui.user_action follow-up, do not render again. Read the fresh parcel value from '
  100 |     + 'interaction.a2uiClientDataModel[interaction.surfaceId].parcel. '
  101 |     + `Return only JSON with marker "${s.marker}" and numeric result equal to that fresh parcel number plus 19.`, instructions: [] };
  102 |   const split = process.env.UAR_MODEL.indexOf('/');
  103 |   assert.ok(split > 0);
  104 |   agent.policy.provider = { default: { provider: process.env.UAR_MODEL.slice(0, split), model: process.env.UAR_MODEL.slice(split + 1) }, fallbacks: [] };
  105 |   agent.policy.tools.allow = ['a2ui_render'];
  106 |   agent.policy.skills.prefer = [];
  107 |   agent.memory.kb.enabled = false;
  108 |   const saved = await api(s, 'POST', '/api/agents', agent, 201);
  109 |   assert.equal(saved.id, s.agentId);
  110 |   await page.goto('/threads', { waitUntil: 'domcontentloaded' });
  111 |   await page.getByRole('button', { name: 'New thread', exact: true }).click();
  112 |   await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  113 |   await page.getByPlaceholder('Search agents...').fill(s.title);
  114 |   await page.getByRole('option').filter({ hasText: s.title }).click();
  115 | });
  116 | 
  117 | When('I ask the agent to render its calculator', async ({ page, scenario: s }) => {
  118 |   const response = observeResponse(page, r => r.request().method() === 'POST' && new URL(r.url()).pathname === '/api/chat/completion');
  119 |   await page.getByRole('textbox', { name: 'Message input', exact: true }).fill('Please render the interactive parcel calculator.');
  120 |   await page.getByRole('button', { name: 'Send message', exact: true }).click();
  121 |   s.initial = await stream(s, await response());
  122 |   const artifacts = s.initial.events.filter(event => event.type === 'CUSTOM' && event.name === 'uar.artifact.available' && event.value?.artifactType === 'a2ui');
> 123 |   assert.equal(artifacts.length, 1, 'Require genuine published A2UI artifact');
      |          ^ AssertionError: Require genuine published A2UI artifact
  124 |   assert.equal(artifacts[0].value.metadata?.sourceTool, 'a2ui_render');
  125 |   assert.equal(artifacts[0].value.metadata?.publication_status, 'published');
  126 |   s.evidence.source_run_id = s.initial.runId;
  127 | });
  128 | 
  129 | Then('the browser displays the generated input and action button', async ({ page, scenario: s }) => {
  130 |   s.surface = page.getByRole('region', { name: 'A2UI display artifact', exact: true });
  131 |   await expect(s.surface.locator('[data-a2ui-surface-state="ready"]')).toBeVisible();
  132 |   await expect(s.surface).toContainText('Parcel calculator');
  133 |   await expect(s.surface.getByRole('textbox', { name: 'Parcel number', exact: true })).toBeVisible();
  134 |   await expect(s.surface.getByRole('button', { name: 'Calculate parcel', exact: true })).toBeVisible();
  135 |   await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'rendered-surface.png'), fullPage: true });
  136 | });
  137 | 
  138 | When('I enter a fresh number and press the rendered calculate button', async ({ page, scenario: s }) => {
  139 |   s.number = randomInt(100, 900);
  140 |   s.evidence.fresh_input = s.number;
  141 |   await s.surface.getByRole('textbox', { name: 'Parcel number', exact: true }).fill(String(s.number));
  142 |   const action = observeResponse(page, r => r.request().method() === 'POST' && /\/api\/uar\/runs\/[^/]+\/a2ui\/actions$/.test(new URL(r.url()).pathname));
  143 |   const continuation = observeResponse(page, r => r.request().method() === 'GET' && /\/api\/uar\/runs\/[^/]+\/stream$/.test(new URL(r.url()).pathname));
  144 |   await s.surface.getByRole('button', { name: 'Calculate parcel', exact: true }).click();
  145 |   const response = await action();
  146 |   const body = response.request().postDataJSON();
  147 |   const ack = await response.json();
  148 |   s.evidence.action = { path: new URL(response.url()).pathname, request: body, status: response.status(), response: ack };
  149 |   assert.equal(response.status(), 202);
  150 |   assert.equal(body.name, 'calculate_parcel');
  151 |   assert.equal(body.a2uiClientDataModel?.[body.surfaceId]?.parcel, String(s.number));
  152 |   assert.equal(ack.status, 'continued');
  153 |   assert.equal(ack.run_id, s.initial.runId);
  154 |   assert.ok(ack.continuation_run_id && ack.continuation_run_id !== ack.run_id);
  155 |   s.continuation = await stream(s, await continuation());
  156 |   assert.equal(s.continuation.runId, ack.continuation_run_id);
  157 | });
  158 | 
  159 | Then('the agent answers using that browser input and its saved instructions', async ({ page, scenario: s }) => {
  160 |   const final = s.continuation.text.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  161 |   let answer;
  162 |   try { answer = JSON.parse(final); } catch {
  163 |     const fences = [...final.matchAll(/(?:^|\n)```(?:json)?[ \t]*\r?\n([\s\S]*?)\r?\n```(?=\s|$)/g)];
  164 |     assert.equal(fences.length, 1);
  165 |     answer = JSON.parse(fences[0][1]);
  166 |   }
  167 |   assert.deepEqual(answer, { marker: s.marker, result: s.number + 19 });
  168 |   const last = page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]').last();
  169 |   await expect(last).toContainText(s.marker);
  170 |   await expect(last).toContainText(String(s.number + 19));
  171 |   s.evidence.answer = answer;
  172 |   s.evidence.continuation_run_id = s.continuation.runId;
  173 |   s.evidence.result = 'VERIFIED';
  174 |   await page.screenshot({ path: resolve(process.env.UAR_BROWSER_EVIDENCE_DIR, 'continued-answer.png'), fullPage: true });
  175 | });
  176 | 
```