# Positive verification — `rewrite-readme-and-docs`

Date: 2026-08-18

## Docusaurus and documentation

Commands:

```bash
cd website
npm ci
npm run typecheck
npm run lint
npm run build
cd ..
pnpm run docs:validate
```

Observed output:

```text
npm ci: added 1409 packages; audited 1410 packages; 20 high severity vulnerabilities reported [exit 0]
typecheck: tsc [exit 0]
lint: Vale CLI is not installed. Skipping prose lint. [exit 0]
Docusaurus 3.10.2 production build: Generated static files in "build". [exit 0]
Documentation truth gate passed (11 canonical files). [exit 0]
```

No Vale-pass or dependency-audit-clean claim is made.

## OpenAPI — UAR server-full on macOS

Commands:

```bash
RUSTC_WRAPPER= cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
RUSTC_WRAPPER= cargo test --locked -p universal-agent-runtime --no-default-features --features server-full spec_uses_package_version_and_documents_customer_routes --lib
```

Observed output:

```text
cargo check: Finished `dev` profile [exit 0]
universal-agent-runtime (lib) generated 3 existing warnings
test uar::api::openapi::tests::spec_uses_package_version_and_documents_customer_routes ... ok
test result: ok. 1 passed; 0 failed; 606 filtered out
```

The test profile also reported the three existing library warnings and one
linker compact-unwind warning. No warning-free claim is made.

## Exact root-hygiene guard

Command:

```bash
for f in TEST_EXECUTION_REPORT.md output.txt output_m3.txt output_m6.txt u00261; do
  test ! -e "$f" || exit 1
done
```

Observed: exit 0 with no output.

## Final candidate hashes

```text
d61207eb8c6a3aab9a924135279a93645ab707bd2ffabe83d70f2c6cbcb4d6f5  README.md
e004b422bfafa12bf3ed0b5b3bd4fa2fe1386cee25df0b567a5f2ec8f71f3ac0  website/docusaurus.config.ts
65bb3daade455141b69568ca11427020a9a851b0b67de0101d015dfc45f5223d  website/package.json
059caf5e4bc2fe6e3b9d8e75033ddfbc2cb859f6a23a99e6d3cd8b245871f31a  website/package-lock.json
0e2b3de44ae26356737291fc9799e78952c35d0ad2950191ed16d834a6d7f293  website/docs/architecture/intro.md
81611286c4699192d8560428e9a5289dfd3212932b5a4baa9f165d3cdea5d976  website/docs/installation.md
894fc9afdb6741357c942d17f601d50b3661f3da8cf388002dea53d08b73adf5  website/docs/sdks.md
4bcb78b645d354035cbdd0dbab9f20995cd7a75ef5d00d3dbfc6418e887f64e0  website/docs/skills.md
51f3367e1c6838f357006fa5cdd3c7c95528e4cd250bffb7a7d74e5d136c5854  website/docs/deployment.md
b65149366524d6ea517b5e28001b8fdc5ce344e643f23c540473546dfea9abce  website/docs/security.md
2a4944775c7f600324b4266535a0ecbacf2f4c0005a10e671be9f26ab0e6f9fc  src/uar/api/openapi.rs
d2a75997c77baa2ad408aca170fe7d3e21f20faeb0f05e28806f6330214312b7  scoped binary diff
```
