import { test, expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
const runtime = process.env.IROH_TEST_RUNTIME || 'runtime';
const peerBinary = process.env.IROH_PEER_BINARY || './target/debug/iroh-lab-peer';
async function invitation() { return JSON.parse(await readFile(`${runtime}/invitation.json`, 'utf8')); }
function encoded(invite) { return Buffer.from(JSON.stringify(invite)).toString('base64url'); }

test('real browser pairs, exchanges messages both ways, pings, and reconnects', async ({ page }) => {
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  const invite = await invitation();
  await page.goto(`./#invite=${encoded(invite)}`);
  await expect(page).not.toHaveURL(/#invite=/);
  await page.getByRole('button', { name: /Connect to sandbox/ }).click();
  await expect(page.locator('#status')).toHaveText('Connected', { timeout: 55_000 });
  await expect(page.locator('#path')).toHaveText('Encrypted · relay');
  await expect(page.locator('#path')).toHaveAttribute('data-kind', 'relay');
  await expect(page.locator('#latency')).toHaveText(/\d+ ms/, { timeout: 25_000 });
  const message = `Phone browser test ${Date.now()} <b>plain text</b>`;
  await page.getByLabel('Message to sandbox').fill(message);
  await page.getByRole('button', { name: 'Send message', exact: true }).click();
  await expect(page.locator('.bubble p').filter({ hasText: message })).toHaveCount(1, { timeout: 25_000 });
  expect(await page.locator('.bubble b').count()).toBe(0);
  const status = JSON.parse(execFileSync(peerBinary, ['--state-dir', runtime, 'status'], { encoding: 'utf8' }));
  expect(status.messages.some(m => m.text === message && m.from === 'phone')).toBe(true);
  const reply = `Sandbox reply ${Date.now()}`;
  execFileSync(peerBinary, ['--state-dir', runtime, 'send', reply]);
  await expect(page.locator('.bubble p').filter({ hasText: reply })).toHaveCount(1, { timeout: 25_000 });
  await page.getByRole('button', { name: 'Disconnect', exact: true }).click();
  await expect(page.locator('#status')).toHaveText('Disconnected');
  await page.getByRole('button', { name: /Connect to sandbox/ }).click();
  await expect(page.locator('#status')).toHaveText('Connected', { timeout: 55_000 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect(errors).toEqual([]);
  await page.getByRole('button', { name: 'Forget invitation' }).click();
  await expect(page.locator('#status')).toHaveText('Ready to pair');
  await expect(page.locator('#fingerprint')).toBeHidden();
  expect(await page.evaluate(() => localStorage.length)).toBe(0);
});

test('wrong invitation token is rejected', async ({ page }) => {
  const invite = await invitation();
  invite.token = invite.token === '0'.repeat(64) ? '1'.repeat(64) : '0'.repeat(64);
  await page.goto(`./#invite=${encoded(invite)}`);
  await page.getByRole('button', { name: /Connect to sandbox/ }).click();
  await expect(page.locator('#status')).toHaveText('Couldn’t connect', { timeout: 55_000 });
  await expect(page.locator('#notice')).toContainText('Invitation rejected');
});

test('PWA manifest, cached offline shell, and malformed invitation recovery', async ({ page, context }) => {
  await page.goto('./');
  await page.locator('#invitation').fill('not an invitation');
  await page.getByRole('button', { name: /Connect to sandbox/ }).click();
  await expect(page.locator('#notice')).toContainText('Paste the full invitation');
  const manifest = await page.evaluate(async () => (await fetch('./manifest.webmanifest')).json());
  expect(manifest.display).toBe('standalone');
  expect(manifest.start_url).toBe('./');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await expect.poll(() => page.evaluate(() => !!navigator.serviceWorker.controller)).toBe(true);
  expect(await page.evaluate(async () => {
    const cache = await caches.open((await caches.keys()).find(k => k.startsWith('iroh-lab:')));
    return !!(await cache.match(new URL('./wasm/iroh_lab_bg.wasm', location.href)));
  })).toBe(true);
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole('heading', { name: /Your sandbox/ })).toBeVisible();
  await context.setOffline(false);
});
