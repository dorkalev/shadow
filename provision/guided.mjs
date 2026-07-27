#!/usr/bin/env node
// guided.mjs — open a console page in a real browser with a floating step overlay,
// then let the human do the sensitive click. Used by atonement.sh for settings
// that have no API (Workspace 2SV, org security). Safe by design: we NEVER click
// the dangerous control for you — we take you to the exact page and tell you what
// to press. Playwright drives navigation; you drive the decision.
//
//   URL=https://... STEPS=$'step 1\nstep 2' node guided.mjs
//
// Requires playwright (npx playwright install chromium once). Falls back cleanly
// if unavailable — atonement.sh prints the URL+steps regardless.
const url = process.env.URL;
const title = process.env.TITLE || 'Shadow — guided step';
const steps = (process.env.STEPS || '').split('\n').filter(Boolean);
if (!url) { console.error('URL env required'); process.exit(2); }

let chromium;
try { ({ chromium } = await import('playwright')); }
catch { console.error('playwright not installed — open this URL manually:\n  ' + url); process.exit(3); }

// Use the system Chrome so existing Google/GitHub logins carry over (no creds handled here).
const browser = await chromium.launch({ channel: 'chrome', headless: false })
  .catch(() => chromium.launch({ headless: false }));
const ctx = await browser.newContext();
const page = await ctx.newContext ? null : await ctx.newPage();

const overlay = `
(() => {
  const d = document.createElement('div');
  d.style.cssText = 'position:fixed;top:16px;right:16px;z-index:2147483647;width:340px;'
    + 'background:#f6f1e4;color:#211c14;border:2px solid #211c14;border-radius:6px;'
    + 'font:13px/1.5 -apple-system,Georgia,serif;box-shadow:0 12px 40px rgba(0,0,0,.4);padding:14px 16px';
  d.innerHTML = ${JSON.stringify(
    `<div style="font-weight:700;border-bottom:1px solid #d5cab0;padding-bottom:6px;margin-bottom:8px">${escapeHtml(title)}</div>`
    + `<ol style="margin:0 0 6px 18px;padding:0">${steps.map(s => `<li style="margin:4px 0">${escapeHtml(s)}</li>`).join('')}</ol>`
    + `<div style="font:11px monospace;color:#9e2b25;border-top:1px solid #d5cab0;padding-top:6px;margin-top:6px">the shadow will not click this for you — you make the change, then close the window</div>`
  )};
  document.documentElement.appendChild(d);
})();`;

function escapeHtml(s){return s.replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));}

const p = page || await ctx.newPage();
await p.goto(url, { waitUntil: 'domcontentloaded' }).catch(() => {});
await p.addInitScript(overlay); // survives in-page navigations
await p.evaluate(overlay).catch(() => {});
p.on('framenavigated', () => p.evaluate(overlay).catch(() => {}));

console.log('Guided window open. Do the steps shown in the overlay, then close the browser.');
await p.waitForEvent('close', { timeout: 0 }).catch(() => {});
await browser.close().catch(() => {});
