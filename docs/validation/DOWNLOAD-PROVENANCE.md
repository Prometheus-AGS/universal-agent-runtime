# Methodology download — 2026-10-27

Source: https://ipfs.prometheusags.ai/ipfs/bafybeihp44qigr3g5zoalkt3p6xvovaglcuzbsp3l4os5vba26glfz6zya/index.html

Downloaded successfully using curl. The web-reader tool could not open it, so the actual HTML was downloaded directly.

| File | SHA256 |
|---|---|
| uar-validation-methodology.original.html | `83becd798cfb1524ec2395907050beb970381b1833d59bff8a7f9270a69480c2` |
| style.css | `045b920b032efd5ad4027af7d71b40ba7f36c32ce3d36e02b579c7b2bd2e68a7` |
| assets/knowme.svg | `4eb3269ac8d41b22dee01b34580d487c565edcb951d78acba960b02874f4ae65` |

At handoff, `uar-validation-methodology.html` is still byte-identical to the original. The requested editorial update is NOT complete. Preserve the original and edit only the new copy. The original refers to app.js/vendor scripts not downloaded; CSS and logo are retained. The revised deliverable should be readable locally and should not depend on missing interaction scripts. C33 is NOT RUN and must not be described as passed until actual evidence exists.

## Revision — 2026-10-27

`uar-validation-methodology.html` was edited on 2026-10-27 into a local revision. It is no longer byte-identical to the original. `uar-validation-methodology.original.html`, `style.css` and `assets/knowme.svg` were not modified. No hash is recorded for the revised file.

What changed in `uar-validation-methodology.html`:

- **Script and missing-file removal.** The `<script>` references to `vendor/htmx-2.0.8.min.js`, `app.js` and `vendor/alpine-3.15.8.min.js` were removed, along with the inline script that added the `js` class. The `manifest.webmanifest` link was also removed. These files were never downloaded. UI that only worked with those scripts was removed: the heading search box, the mobile Contents bar, and the htmx `edition.html` loader. The `Markdown ↓` link pointed to an undownloaded file and was replaced with an in-page link to the revision notes. Inert Alpine attributes remain on some elements. Without JavaScript all content is visible, including the contents list on narrow screens. `style.css` and `assets/knowme.svg` stay as local relative links. A small inline `<style>` block styles the revision notes using the existing CSS variables. The web fonts that `style.css` references under `assets/fonts/` were not downloaded, so the page falls back to system fonts.
- **Revision header.** The title, top bar edition line and edition box are marked "Revised 2026-10-27". A revision banner after the edition box links the source IPFS URL above and this file.
- **Visible "Revised:" notes.** Where later evidence supersedes a statement, a note was added next to it and the original text was kept. The affected places are Section 2.2 (the 97-case × two-store × three-repetition plan), Section 2.3 (the packet-and-approval relapse), Section 2.5 (the 25-label / 22-journey cutoff count), Section 8.5 (knowledge-restart not established), Section 9.2 (the packet/single-run contract), Case evidence C7, and the footer (independent-review statement).
- **Lessons section.** New Section 20, "Lessons from manual Mac verification (October 2026)", subsections 20.1–20.15. Each lesson cites repository sources under `.prometheus/` and `scripts/behaviors/`. It records the accepted counts of 35 labels / 32 outcomes and 31 BDD reports + 1 standalone prompt, with B03/C03 as composite evidence. It also records the proposed 102 / 32 / 70 catalog and its platform and backend residuals.
- **C33 placeholder.** An empty `<section id="c33-result">` with the heading "C33 — REST model configuration is effective and durable (Mac)" and `<p data-c33-placeholder>Result pending.</p>`. No C33 result is recorded. C33 had not run when this revision was written.

No test, build, validator or rendering check was run to produce this revision.

### C33 result added — 2026-10-27

After C33 actually ran and passed on Mac, the lead filled `#c33-result` with the observed result and its limits. The lead also updated the revision banner and lesson 20.8, which had said C33 was not yet run. The original file remains unchanged.
