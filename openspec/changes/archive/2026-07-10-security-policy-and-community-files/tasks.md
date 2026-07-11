## 1. Policy files

- [x] 1.1 SECURITY.md (manufacturer posture, private reporting, targets,
      supported versions, CRA note). Already landed in 0652d79; verified
      complete (GitHub private vuln reporting channel + CRA manufacturer note).
- [x] 1.2 SUPPORT.md. Already landed in 0652d79.
- [x] 1.3 .github/ISSUE_TEMPLATE/ (bug.yml, feature.yml, config.yml routing
      security to private reporting). Already landed in 0652d79; config.yml
      routes to security/advisories/new.
- [x] 1.4 Licensing doc exists at docs/licensing/LICENSING.md (README already
      links it in the Licensing section). Added the missing README **Security**
      section linking SECURITY.md + SUPPORT.md + private-reporting URL.

## 2. Enablement + bookkeeping

- [x] 2.1 GitHub private vulnerability reporting ENABLED on the repo
      (gh api PUT; GET returns {"enabled":true}).
- [x] 2.2 Commit, push, archive; update phase state. (openspec CLI not
      installed locally — validated change structure manually: proposal +
      spec delta + tasks intact; archived via git mv per repo convention.)
