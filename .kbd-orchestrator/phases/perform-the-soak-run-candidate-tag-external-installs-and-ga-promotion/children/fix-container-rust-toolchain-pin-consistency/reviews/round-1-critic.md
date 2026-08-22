# Artifact Critic — Round 1

Verdict: **BLOCK**

1. Source-bound evidence was circular: a committed `verification.md` cannot
   contain the SHA of the commit that contains it. Define a two-commit
   sequence, external evidence artifact, or non-self-referential source-tree
   digest.
2. The fail-closed contract did not cover an effective Docker build-argument
   override that differs from the Dockerfile default.
3. ARM64 verification did not name the target platform, execution environment,
   exact commands, or immutable incompatible nightly.
4. The specification governed every Rust compilation while the proposed
   validation covered only the observed backend build command.

Non-blocking: the one-selector product edit and narrow local check are minimal;
the layered compiler and full-image evidence is useful; stopping on a newly
exposed unrelated image failure prevents scope creep.
