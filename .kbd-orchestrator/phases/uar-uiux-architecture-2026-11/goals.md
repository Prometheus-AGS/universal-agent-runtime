# Goals

- PGlite architecture and client-side state using Prometheus Entity Management
- Run the Impeccable CLI to update and initialize, then use the Impeccable skills to evaluate and improve the UI/UX
- Persist every configuration setting in the database and use it from there, kept in sync with changes to the YAML configuration files without any risk of rendering the system unusable
- Change LLM providers and models at the system, agent, agent team and conversation levels in realtime, with the effect realized immediately in conversations
- Remove UI/UX in the application or settings that does not make sense
- Everywhere an LLM or embedding model and provider is selected, give the user a dropdown or popup to choose from; never expect the user to type the full provider/model name
- Handle ALL settings that way; the user types text only for genuine text fields such as prompts, essay-like answers and content
- Use web workers to improve the user experience only where it makes sense and has no undesirable side effects
- Deliver an S-tier professional user experience: elegant, minimalist design with selective discovery that lets users grow into more choices as needed; the current UI is too complex and requires too much typing
