export function parseInvitation(input) {
  if (typeof input !== 'string' || input.length > 8192) throw new Error('That invitation is too long.');
  let encoded = input.trim();
  if (encoded.includes('#')) encoded = new URLSearchParams(encoded.slice(encoded.indexOf('#') + 1)).get('invite') || '';
  else if (encoded.startsWith('invite=')) encoded = encoded.slice(7);
  if (!/^[A-Za-z0-9_-]+$/.test(encoded)) throw new Error('Paste the full invitation link from the sandbox.');
  let invite;
  try {
    const bytes = Uint8Array.from(atob(encoded.replaceAll('-', '+').replaceAll('_', '/')), c => c.charCodeAt(0));
    invite = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  } catch { throw new Error('This invitation could not be read. Ask for a fresh link.'); }
  if (invite.version !== 1 || !/^[0-9a-f]{64}$/i.test(invite.endpoint_id) || !/^[0-9a-f]{64}$/i.test(invite.token)) throw new Error('This invitation is invalid or uses an unsupported version.');
  let relay;
  try { relay = new URL(invite.relay_url); } catch { throw new Error('This invitation has an invalid relay address.'); }
  if (relay.protocol !== 'https:' || relay.username || relay.password || invite.relay_url.length > 2048) throw new Error('This invitation needs a secure relay address.');
  return { version: 1, endpoint_id: invite.endpoint_id, relay_url: invite.relay_url, token: invite.token };
}
export function fingerprint(id) { return `${id.slice(0, 8)} … ${id.slice(-8)}`; }
