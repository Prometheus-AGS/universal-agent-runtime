# Spec adversarial review — 2026-10-17

Fresh-context native `uar-verifier`; artifact-only source critique. No product
verification ran. Gate **CLOSED**. Verdict: no critical, two warnings and one
suggestion. Producer model/family separation is unverified.

- The output oracle allowed prompt-only dependence. Spec now requires exact
  response equality with a deterministic function of both received prompt and
  skill markers across public output and durable readback.
- “Materially slow” lacked an owner and threshold. Spec now records diagnostic
  timing and requires operator disposition of observations before release,
  without inventing a numeric pass claim.
- Spec now states 65 local labels and ten labels on each published child, with
  stage-specific group membership; omitted labels cannot be credited.

The existing OpenSpec `verification.md` still says six groups per published
child. Updating it is an explicit FC1/FC4 implementation task, so the new
scope is not yet active. No runtime behavior is certified.
