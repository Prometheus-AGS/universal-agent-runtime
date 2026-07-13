# Agent work estimation rule

Decision date: 2026-07-12
Decision authority: project operator

All implementation estimates must assume that the work will be performed
autonomously through an agent harness such as Codex, Claude Code, or OpenCode,
using a current high-capability coding model in the GPT-5.6, Claude Sonnet 5,
GLM 5.2, Kimi K2.7 Coding, or MiniMax M3 class.

Estimates must therefore:

- use agent execution time rather than human engineering velocity;
- separate active agent-hours from elapsed time spent on builds, CI, external
  approvals, soak tests, and other intrinsically time-bound gates;
- assume safe parallel execution when tasks have independent ownership;
- include verification and likely debugging, but not human-paced typing,
  scheduling, handoff, or review-cycle delays unless the task explicitly
  requires human participation;
- state assumptions and give a range when repository coupling or external
  infrastructure creates material uncertainty.

This is the default estimation model unless the operator explicitly requests a
human-team estimate or specifies a different harness/model class.
