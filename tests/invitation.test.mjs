import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseInvitation } from '../web/invitation.js';
const sample = { version: 1, endpoint_id: 'a'.repeat(64), relay_url: 'https://relay.example/', token: 'b'.repeat(64) };
const encode = obj => Buffer.from(JSON.stringify(obj)).toString('base64url');
test('accepts a full URL, fragment, or raw token', () => {
  const encoded = encode(sample);
  for (const text of [encoded, `#invite=${encoded}`, `https://example.org/lab/#invite=${encoded}`]) assert.deepEqual(parseInvitation(text), sample);
});
test('rejects malformed, oversized, unsupported, and insecure invitations', () => {
  for (const text of ['<script>', 'a'.repeat(9000), encode({ ...sample, version: 2 }), encode({ ...sample, token: 'bad' }), encode({ ...sample, relay_url: 'http://relay.example/' }), encode({ ...sample, relay_url: 'https://user:pass@relay.example/' })]) assert.throws(() => parseInvitation(text));
});
