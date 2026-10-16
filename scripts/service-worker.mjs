import { readdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
async function walk(dir) {
  const out = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = `${dir}/${entry.name}`;
    if (entry.isDirectory()) out.push(...await walk(path));
    else if (!['sw.js', '.nojekyll'].includes(entry.name)) out.push(path);
  }
  return out.sort();
}
const files = await walk('dist');
const hash = createHash('sha256');
for (const file of files) { hash.update(file); hash.update(await readFile(file)); }
const version = hash.digest('hex').slice(0, 16);
await writeFile('dist/sw.js', `// Generated from all application assets. Do not edit.
const PREFIX = 'iroh-lab:' + self.registration.scope;
const CACHE = PREFIX + '${version}';
const FILES = ${JSON.stringify(files.map(file => './' + file.slice(5)))};
const ASSETS = new Set(FILES.map(path => new URL(path, self.registration.scope).href));
self.addEventListener('install', event => event.waitUntil(caches.open(CACHE).then(cache => cache.addAll(FILES))));
// Let existing tabs finish on their old version; never force a mixed JS/Wasm update.
self.addEventListener('activate', event => event.waitUntil((async () => {
  for (const key of await caches.keys()) if (key.startsWith(PREFIX) && key !== CACHE) await caches.delete(key);
  await self.clients.claim();
})()));
self.addEventListener('fetch', event => {
  if (event.request.method !== 'GET') return;
  const url = new URL(event.request.url);
  if (url.origin !== self.location.origin) return;
  if (event.request.mode === 'navigate' && url.href.startsWith(self.registration.scope)) {
    event.respondWith(caches.open(CACHE).then(cache => cache.match(new URL('./index.html', self.registration.scope))).then(response => response || fetch(event.request)));
  } else if (ASSETS.has(url.href)) {
    event.respondWith(caches.open(CACHE).then(cache => cache.match(event.request)).then(response => response || fetch(event.request)));
  }
});
`);
await writeFile('dist/.nojekyll', '');
console.log(`Built PWA ${version}; ${files.length} static assets.`);
