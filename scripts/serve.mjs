import http from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { resolve, extname } from 'node:path';
const root = resolve('dist');
const types = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.webmanifest': 'application/manifest+json', '.svg': 'image/svg+xml', '.png': 'image/png' };
http.createServer(async (req, res) => {
  try {
    let path = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    if (path.startsWith('/iroh-lab/')) path = path.slice('/iroh-lab'.length);
    let file = resolve(root, '.' + path);
    if (!file.startsWith(root + '/') && file !== root) { res.writeHead(403); res.end(); return; }
    if ((await stat(file)).isDirectory()) file += '/index.html';
    res.writeHead(200, { 'Content-Type': types[extname(file)] || 'application/octet-stream', 'Cache-Control': 'no-cache' });
    res.end(await readFile(file));
  } catch { res.writeHead(404); res.end('Not found'); }
}).listen(Number(process.env.PORT || 4173), '127.0.0.1', () => console.log('PWA: http://127.0.0.1:4173/iroh-lab/'));
