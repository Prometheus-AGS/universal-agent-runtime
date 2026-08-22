# Refinement log — `ship-skill-pack-install-path`

## Iteration 1 — 2026-08-18T17:51:31Z

- Specify: derived the public source, verified build, atomic install, inventory,
  and documentation constraints from C5 and the OpenSpec change.
- Plan: retain UAR's existing installed-plugin precedence and add one external
  installer that produces its versioned cache layout.
- Execute: switched the parent submodule to HTTPS; added the pinned installer,
  deterministic shell controls, clean-prefix API integration test, and guide.
- Reflect: moved staging outside the loader scan after identifying a concurrent-
  startup visibility race; no runtime code change was required.
- Observe: public pin fetch passed; the real locked release build/install passed
  in 4m53s; shell controls passed; API inventory passed 1/0; Tier 0 and strict
  OpenSpec passed; scoped Clippy exited 0 with 572 warnings.
- Persist: wrote OpenSpec and PMPO receipts. Independent artifact critic and
  judge remain the termination gate.
- Content hashes: gitmodules `6a6ffac689d017453fb30031f739555dcb5e136a00679142d5bfcfd24329e8a2`;
  installer `d9c2843dc4b135c8cd8b355486a3e1777d3d9282ac14ef7a5ee192ae9ec425f4`;
  shell test `cb0b085c756eb4f0c0d04b2cb453787d0c20c34fddc1dcdd156b33283d41bbc9`;
  API test `7fab5da3c9b6bad90ab201e589e582bd687adb408f6e7c6c34628c211d50eb6a`;
  guide `9f1b82c9d5b48049277cdadf45665c7125307f179b8756598e40c36837f27c30`.

## Iteration 2 — 2026-08-18T17:59:54Z

- Reflect: independent criticism found that the installer honored XDG while the
  runtime did not, and that the API test did not name the default imported-skill
  boundary or require an exact count.
- Execute: aligned the default install root with the runtime's existing HOME
  path; preserved imported skills as explicit opt-in; changed the clean-prefix
  test to require exactly 147 default-eligible skills and exact API IDs.
- Observe: Bash syntax and ShellCheck exited 0; Tier 0 exited 0 with three known
  warnings; the corrected focused test passed 1/0 with 147 discovered skills.
- Persist: amended OpenSpec, operator documentation, verification receipts, and
  artifact constraints to distinguish 311 copied manifests from 147 skills in
  the pinned pack's default loader inventory.
- Current content hashes: gitmodules `6a6ffac689d017453fb30031f739555dcb5e136a00679142d5bfcfd24329e8a2`;
  installer `68db85c0bc1a6de2626764957c3d85fb75b671a852710d84e26e17bad7a39aa6`;
  shell test `cb0b085c756eb4f0c0d04b2cb453787d0c20c34fddc1dcdd156b33283d41bbc9`;
  API test `a2f4a7c698f60e34146dc854300d1d1c28d74b52c349579685fa4c37a01525b7`;
  guide `5dc0c17405ec891d01e6743958fc56f4861e577f538c9171e2899afa0f361659`.
- Reflect: independent critic and judge both accepted the corrected candidate.
  Their only remaining warning is to exclude unrelated operator and generated
  files from the explicit commit.
