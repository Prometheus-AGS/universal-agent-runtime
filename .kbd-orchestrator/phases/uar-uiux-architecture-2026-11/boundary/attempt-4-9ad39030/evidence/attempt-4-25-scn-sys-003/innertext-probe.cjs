// Standalone browser probe: no UAR code, page, server or test is loaded.
const { createRequire } = require('node:module');
const req = createRequire('/Volumes/T7/Projects/prometheus/universal-agent-runtime/package.json');
const { chromium } = req('@playwright/test');
const turn = (i, H) => `<p id="u${i}">user ${i}</p><div class="a" id="a${i}"><div style="height:${H}px">policy ${i}</div><p>bdd-reply-model-${i}</p><p>Answered by</p></div>`;
(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1000, height: 600 } });
  for (const H of [700, 1500, 3000]) for (const style of ['content-visibility:auto;contain-intrinsic-size:auto 200px', '']) {
    await page.setContent(`<style>.a{${style}}#v{height:600px;overflow-y:auto}</style><section><div id="v">${turn(1, H)}${turn(2, H)}<div id="more"></div><div style="height:600px"></div></div></section>`);
    const count = () => page.evaluate(() => {
      const n = (t) => (t.match(/bdd-reply-[A-Za-z0-9._-]+/g) || []).length;
      const s = document.querySelector('section');
      const w = document.createTreeWalker(s, NodeFilter.SHOW_TEXT);
      const parts = [];
      while (w.nextNode()) parts.push(w.currentNode.nodeValue);
      return { firstAnswerSkipped: !document.querySelector('#a1 p').checkVisibility({ contentVisibilityAuto: true }), innerText: n(s.innerText), textNodes: n(parts.join('\n')) };
    });
    const settle = () => page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(r, 400)))));
    const to = async (id) => { await page.evaluate((i) => { const v = document.querySelector('#v'); v.scrollTop += document.getElementById(i).getBoundingClientRect().top - v.getBoundingClientRect().top; }, id); await settle(); };
    await to('u1'); await to('u2');
    const before = await count();
    await page.evaluate((h) => { document.querySelector('#more').innerHTML = h; }, turn(3, H));
    await settle(); await to('u3');
    console.log(JSON.stringify({ height: H, contentVisibility: style ? 'auto' : 'none', before, after: await count() }));
  }
  await browser.close();
})().catch((e) => { console.error(String(e)); process.exit(1); });
