## Purpose

Defines what each configurable skill-matching algorithm does, which model or embedding backend it may use, how it fails, how the method actually used is reported, and the legacy status of the intent-classifier settings namespace.

## ADDED Requirements

### Requirement: LLM skill matching ranks skills with the run's model binding
When the skill-matching algorithm is `llm`, the runtime SHALL score eligible skills by asking a language model to rank them for the user input. The call SHALL use the model binding the run captured, with its credentials and budget, and SHALL NOT construct a new model client. When `model_name` is set, the call SHALL use that model by rebinding the captured client within its credential grant. The reply's skill ids SHALL be intersected with the eligible set, and scores SHALL be limited to the range 0.0 to 1.0; ids outside the eligible set and non-numeric scores SHALL be discarded. The existing threshold, margin and `top_k` rules SHALL apply to the resulting scores.

#### Scenario: Paraphrase with no keyword overlap
- **WHEN** the algorithm is `llm` and the input paraphrases a skill's purpose without any of its keywords, and the model ranks that skill above the threshold
- **THEN** that skill is the accepted match, although keyword matching accepts no skill for the same input

#### Scenario: Model name is honoured
- **WHEN** `model_name` names a model reachable under the run's captured provider grant
- **THEN** the classification request is sent through the run's captured client with that model

#### Scenario: Reply names an ineligible skill
- **WHEN** the model's reply includes a disabled skill, an unknown skill id and a score outside 0.0 to 1.0
- **THEN** the disabled and unknown ids are not matched and the out-of-range score is limited or discarded

### Requirement: LLM skill matching falls back to keyword matching visibly
When the classification call fails, its reply is not parseable, or `model_name` cannot be bound under the run's captured grant, the runtime SHALL match with the keyword algorithm for that turn and SHALL report the fallback as the selection method. The fallback SHALL NOT fail the run.

#### Scenario: Malformed reply
- **WHEN** the algorithm is `llm` and the model replies with text that is not the expected JSON object
- **THEN** the match result equals the keyword result for the same input and the reported selection method is the LLM keyword fallback

#### Scenario: Model name cannot be bound
- **WHEN** `model_name` names a model the run's captured client cannot rebind to
- **THEN** no classification request is sent, keyword matching is used, and the fallback is reported

### Requirement: Under LLM matching the run's model binding is fixed before matching
When the algorithm is `llm`, a root run SHALL capture its model binding before skill matching and SHALL NOT capture it again afterwards. A matched skill's `preferred_model` SHALL be applied only by rebinding the captured client within its credential grant. A `preferred_model` under a provider outside that grant SHALL be ignored for the run, and the runtime SHALL record that it was ignored. Under every other algorithm the order of binding capture and skill matching SHALL be unchanged.

#### Scenario: Cross-provider preferred model under llm
- **WHEN** the algorithm is `llm` and the matched skill's `preferred_model` names a provider the run's captured grant does not cover
- **THEN** the run executes on its captured binding, the preferred model receives no request, and the ignored preference is recorded

#### Scenario: Same-provider preferred model under llm
- **WHEN** the algorithm is `llm` and the matched skill's `preferred_model` names a model under the captured provider
- **THEN** the run executes on the captured client rebound to that model

### Requirement: Local-embedding skill matching runs on the device
When the algorithm is `local_embedding`, the runtime SHALL score skills by cosine similarity between an on-device embedding of the input and cached on-device embeddings of each eligible skill. It SHALL NOT call the configured shared embedding backend, SHALL work with persistence disabled, and SHALL rebuild its skill-vector cache when skills are loaded, refreshed or reconciled. A skill changed after the last rebuild SHALL NOT be scored with a stale vector.

#### Scenario: No remote embedding calls
- **WHEN** the algorithm is `local_embedding` and three inputs are matched with persistence enabled
- **THEN** the shared embedding backend receives zero embedding requests

#### Scenario: Persistence off
- **WHEN** the runtime has no persistence layer and the algorithm is `local_embedding`
- **THEN** a paraphrased input still matches the expected skill

#### Scenario: Skill changed by reconciliation
- **WHEN** reconciliation changes a skill's description and the next input matches only the new description
- **THEN** that skill is matched using a vector computed from the new description

### Requirement: Local-embedding matching degrades to keyword, never to the remote backend
When the on-device model is not available in the build or its assets cannot be loaded, the `local_embedding` algorithm SHALL match with keyword matching and report the fallback as the selection method. It SHALL NOT fall back to the shared embedding backend.

#### Scenario: Local model unavailable
- **WHEN** the algorithm is `local_embedding` and no on-device embedding model can be loaded
- **THEN** keyword matching is used, the fallback is reported, and the shared embedding backend receives zero requests

### Requirement: The configured algorithm determines which skills activate
A change to the skill-matching algorithm through the skill configuration API SHALL change the skill scores used by the next run, and therefore which skills that run activates or suggests, without a restart. The selection method reported for each activation SHALL name the algorithm or fallback that produced the match.

#### Scenario: Switching from keyword to llm
- **WHEN** a run in legacy overlay mode activates skill A by keyword, the algorithm is then set to `llm` through the configuration API, and the model ranks skill B first for the same input
- **THEN** the next run activates skill B instead of skill A and its activation event reports the LLM selection method

#### Scenario: Switching from keyword to local_embedding
- **WHEN** the algorithm is changed from `keyword` to `local_embedding` and the input paraphrases skill B
- **THEN** the next run activates skill B and its activation event reports the local-embedding selection method

### Requirement: The intent-classifier namespace is legacy
The `intent_classifier` settings namespace and its backends SHALL be reported as legacy in the settings schema, with a description stating that production runs use the skill-matching configuration instead. The runtime SHALL keep accepting reads and writes to the namespace unchanged.

#### Scenario: Settings schema lists the namespace
- **WHEN** a client reads the settings schema for `intent_classifier`
- **THEN** the schema carries the legacy marker and the description that names the skill-matching configuration as the effective control
