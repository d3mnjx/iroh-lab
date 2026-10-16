import { parseInvitation, fingerprint } from './invitation.js';
const $ = id => document.getElementById(id);
let invitation = null, peer = null, generation = 0, cursor = 0, sending = false, pinging = false;
let phase = 'idle', wasmPromise, deferredInstall;
const shown = new Set();

function notice(message = '') { $('notice').textContent = message; $('notice').hidden = !message; }
function setState(state, title, detail) {
  phase = state;
  $('status-pill').dataset.state = state;
  $('status').textContent = title;
  $('status-detail').textContent = detail;
  const connected = state === 'connected';
  $('connect').disabled = state === 'connecting' || connected;
  $('connect').textContent = state === 'connecting' ? 'Connecting…' : connected ? 'Connected ✓' : invitation ? 'Connect to sandbox ↗' : 'Connect to sandbox ↗';
  $('disconnect').disabled = !connected && state !== 'connecting';
  $('ping').disabled = !connected;
  $('message').disabled = !connected;
  $('send').disabled = !connected || sending;
  $('invitation').disabled = state === 'connecting' || connected;
  $('path').textContent = '—';
  if (connected) updatePath();
}
function updatePath() {
  if (!peer) return;
  const kind = peer.path_kind();
  $('path').textContent = kind === 'relay' ? 'Encrypted · relay' : kind === 'direct' ? 'Encrypted · direct' : 'Checking…';
  $('path').dataset.kind = kind;
}
function clearMessages() {
  shown.clear(); cursor = 0;
  $('messages').querySelectorAll('.bubble').forEach(el => el.remove());
  $('empty-state').hidden = false;
  $('message-count').textContent = '0 messages';
}
function showMessages(messages) {
  for (const message of messages) {
    if (shown.has(message.id)) continue;
    shown.add(message.id);
    const wrapper = document.createElement('article');
    wrapper.className = `bubble ${message.from === 'phone' ? 'phone' : 'sandbox'}`;
    wrapper.dataset.id = message.id;
    const meta = document.createElement('div'); meta.className = 'bubble-meta';
    meta.textContent = `${message.from === 'phone' ? 'Phone' : 'Sandbox'} · ${new Date(message.at_ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
    const body = document.createElement('p'); body.textContent = message.text;
    wrapper.append(meta, body);
    const later = [...$('messages').querySelectorAll('.bubble')].find(el => Number(el.dataset.id) > message.id);
    $('messages').insertBefore(wrapper, later || null);
  }
  while ($('messages').querySelectorAll('.bubble').length > 200) $('messages').querySelector('.bubble').remove();
  // Bound deduplication state alongside the visible history.
  if (shown.size > 400) { const cutoff = Math.max(...shown) - 250; for (const id of shown) if (id < cutoff) shown.delete(id); }
  $('empty-state').hidden = shown.size > 0;
  const count = $('messages').querySelectorAll('.bubble').length;
  $('message-count').textContent = `${count} message${count === 1 ? '' : 's'}`;
  if (messages.length) $('messages').scrollTop = $('messages').scrollHeight;
}
async function release(oldPeer) {
  if (!oldPeer) return;
  oldPeer.disconnect();
  try { await oldPeer.close(); } catch { /* Already closed. */ }
}
function stop({ forget = false } = {}) {
  generation++;
  const old = peer; peer = null; sending = false; pinging = false;
  void release(old);
  $('latency').textContent = '—';
  if (forget) {
    invitation = null; $('invitation').value = ''; $('peer-preview').hidden = true;
    $('message').value = ''; clearMessages(); notice();
  }
  setState('idle', invitation ? 'Disconnected' : 'Ready to pair', invitation ? 'Your invitation stays here until you close or reload this page.' : 'Keep the sandbox running while you connect.');
}
function preview(invite) {
  invitation = invite;
  $('fingerprint').textContent = fingerprint(invite.endpoint_id);
  $('peer-preview').hidden = false;
  $('invitation').value = '';
  $('invitation').placeholder = 'Invitation loaded. Ready to connect.';
}
async function loadWasm() {
  if (!wasmPromise) wasmPromise = import('./wasm/iroh_lab.js').then(async module => { await module.default(); return module; }).catch(error => { wasmPromise = null; throw error; });
  return wasmPromise;
}
async function connect(event) {
  event?.preventDefault();
  if (phase === 'connecting' || phase === 'connected') return;
  notice();
  try { if ($('invitation').value.trim()) preview(parseInvitation($('invitation').value)); if (!invitation) throw new Error('Open or paste an invitation from the sandbox first.'); }
  catch (error) { notice(error.message); return; }
  const current = ++generation;
  clearMessages(); $('latency').textContent = '—';
  setState('connecting', 'Connecting', 'Opening your encrypted connection. This can take a few moments.');
  let candidate;
  try {
    const { BrowserPeer } = await loadWasm();
    if (current !== generation) return;
    candidate = await BrowserPeer.connect(JSON.stringify(invitation));
    if (current !== generation) { await release(candidate); return; }
    peer = candidate;
    setState('connected', 'Connected', 'Your phone and sandbox can now exchange messages.');
    void ping();
    void receive(current, candidate);
  } catch (error) {
    if (candidate) void release(candidate);
    if (current !== generation) return;
    peer = null;
    setState('error', 'Couldn’t connect', 'Check the sandbox and invitation, then try again.');
    notice(error.message || 'Connection failed. Please try again.');
  }
}
function connectionFailed(error, current) {
  if (current !== generation) return;
  const old = peer; peer = null; generation++; sending = false; pinging = false;
  void release(old);
  setState('error', 'Connection interrupted', 'Reconnect when both devices are online.');
  notice(error.message || 'The connection closed. Try reconnecting.');
}
async function receive(current, activePeer) {
  while (current === generation && peer === activePeer) {
    try {
      const result = JSON.parse(await activePeer.poll(cursor));
      if (current !== generation) return;
      if (result.history_lost) notice('Some earlier messages have expired from this temporary session.');
      updatePath();
      showMessages(result.messages);
      cursor = result.cursor;
    } catch (error) { connectionFailed(error, current); return; }
  }
}
async function ping() {
  if (!peer || pinging) return;
  const current = generation, activePeer = peer;
  pinging = true; $('ping').disabled = true;
  const start = performance.now();
  try {
    await activePeer.ping();
    if (current === generation) { $('latency').textContent = `${Math.round(performance.now() - start)} ms`; updatePath(); }
  } catch (error) { connectionFailed(error, current); }
  finally { if (current === generation) { pinging = false; $('ping').disabled = !peer; } }
}
$('pair-form').addEventListener('submit', connect);
$('disconnect').addEventListener('click', () => stop());
$('forget').addEventListener('click', () => stop({ forget: true }));
$('ping').addEventListener('click', ping);
$('send-form').addEventListener('submit', async event => {
  event.preventDefault();
  if (!peer || sending) return;
  const text = $('message').value.trim();
  if (!text) return;
  if (new TextEncoder().encode(text).length > 4096) { notice('Please keep the message below 4 KB.'); return; }
  const current = generation, activePeer = peer;
  sending = true; $('send').disabled = true; notice();
  try {
    const result = JSON.parse(await activePeer.send_message(text));
    if (current !== generation) return;
    showMessages(result.messages);
    // Poll owns its cursor so an acknowledgement cannot skip intervening messages.
    $('message').value = '';
  } catch (error) {
    connectionFailed(error, current);
    notice('Delivery was not confirmed. Your text is still in the box. Reconnect and check the conversation before sending it again.');
  } finally { if (current === generation) { sending = false; $('send').disabled = !peer; } }
});
$('message').addEventListener('keydown', event => {
  if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) { event.preventDefault(); $('send-form').requestSubmit(); }
});
window.addEventListener('offline', () => { if (peer || phase === 'connecting') { stop(); notice('You’re offline. The app is available, but messaging needs a network connection.'); } });
window.addEventListener('online', () => { if (!peer) notice('You’re back online. Reconnect to resume your conversation.'); });
document.addEventListener('visibilitychange', () => { if (!document.hidden && peer) void ping(); });
window.addEventListener('pagehide', () => stop());
window.addEventListener('beforeinstallprompt', event => { event.preventDefault(); deferredInstall = event; $('install').hidden = false; });
$('install').addEventListener('click', async () => { if (!deferredInstall) return; await deferredInstall.prompt(); deferredInstall = null; $('install').hidden = true; });
window.addEventListener('appinstalled', () => { $('install').hidden = true; });
if (location.hash) {
  const fragment = location.hash;
  history.replaceState(null, '', location.pathname + location.search);
  try { preview(parseInvitation(fragment)); notice('Invitation loaded. Connect when your sandbox is ready.'); }
  catch (error) { notice(error.message); }
}
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('./sw.js').then(registration => {
    const updateNotice = () => notice('An app update is ready. Disconnect and close all app tabs, then reopen to use it.');
    if (registration.waiting) updateNotice();
    registration.addEventListener('updatefound', () => registration.installing?.addEventListener('statechange', () => { if (registration.waiting && navigator.serviceWorker.controller) updateNotice(); }));
  }).catch(() => notice('Offline setup could not finish. The app can still connect while online.'));
}
