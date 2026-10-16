import { spawn, execFileSync } from 'node:child_process';
import { mkdirSync, openSync, closeSync, writeFileSync, readFileSync } from 'node:fs';
import { setTimeout } from 'node:timers/promises';
process.umask(0o077);
mkdirSync('runtime', { recursive: true, mode: 0o700 });
const binary = process.env.IROH_PEER_BINARY || './target/debug/iroh-lab-peer';
try {
  execFileSync(binary, ['status'], { stdio: 'pipe', timeout: 10_000 });
  console.log('Sandbox peer is already running. Private invitation: runtime/pairing-url.txt');
  process.exit(0);
} catch { /* No responding peer. */ }
let oldToken;
try { oldToken = JSON.parse(readFileSync('runtime/control.json')).token; } catch { /* First run. */ }
const output = openSync('runtime/peer.log', 'a', 0o600);
const child = spawn(binary, ['serve'], { detached: true, stdio: ['ignore', output, output] });
closeSync(output);
let exited = false;
child.on('error', error => { console.error(error.message); exited = true; });
child.on('exit', () => { exited = true; });
for (let n = 0; n < 180; n++) {
  await setTimeout(250);
  if (exited) throw new Error('Peer exited. Read runtime/peer.log for details.');
  try {
    const control = JSON.parse(readFileSync('runtime/control.json'));
    if (control.token !== oldToken) {
      execFileSync(binary, ['status'], { stdio: 'pipe', timeout: 10_000 });
      writeFileSync('runtime/peer.pid', String(child.pid), { mode: 0o600 });
      child.unref();
      console.log('Sandbox peer is ready. Private invitation: runtime/pairing-url.txt');
      process.exit(0);
    }
  } catch { /* Still establishing relay connectivity. */ }
}
child.kill('SIGINT');
throw new Error('Peer did not become ready. Read runtime/peer.log.');
