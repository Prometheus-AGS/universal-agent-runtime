## 1. Provider Default Projection

- [x] 1.1 Project both the configured default provider id and its default model into normalized provider metadata, then verify the frontend typecheck succeeds
- [x] 1.2 Expose a typed provider-domain hook that loads and classifies default-route availability without component-owned business state, then verify lint and typecheck succeed

## 2. Agent Status Explanation

- [x] 2.1 Render truthful loading, inherited-default, unresolved, and registry-failure indicators in the Agents list with row-level hover/focus tooltips, then verify lint and typecheck succeed

## 3. Completed-UI Verification and Deployment

- [x] 3.1 After implementation is code-complete, run strict OpenSpec validation and the production frontend build and record their observed output
- [x] 3.2 Install the completed bundle into the local LaunchAgent and use a real browser to verify the configured agents show the inherited `kimi-for-coding/k3` explanation on pointer hover and keyboard focus at `http://localhost:1906`
- [x] 3.3 Record row-form verification evidence, mark tasks complete only for observed results, and confirm no backend, provider configuration, or unrelated files changed
