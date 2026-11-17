/*
 * WhatsApp-Matter — cross-language authenticated control bridge tests.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  bridgeConfiguration, ownerControlPayload, signControlFrame, privateControlText,
} from '../src/bridge.mjs';

const OWNER = '5215512345678@s.whatsapp.net';
const KEY = Buffer.from(Array.from({ length: 32 }, (_, i) => i + 1));
const CONFIG = {
  socketId: 'socket-1', ownerActorId: 'owner', ownerJid: OWNER, key: KEY, port: 39819,
};

function message(text = '=/>status', id = 'msg-1') {
  return {
    key: { id, fromMe: true, remoteJid: OWNER },
    message: { conversation: text },
  };
}

test('all bridge fields required; no default secret or non-loopback address', () => {
  assert.equal(bridgeConfiguration({}), null);
  const env = {
    WA_BRIDGE_ADDR: '127.0.0.1:39819',
    WA_BRIDGE_KEY_HEX: KEY.toString('hex'),
    WA_BRIDGE_SOCKET_ID: 'socket-1',
    WA_BRIDGE_OWNER_ID: 'owner',
    WA_BRIDGE_OWNER_JID: OWNER,
  };
  assert.equal(bridgeConfiguration(env).ownerJid, OWNER);
  assert.throws(() => bridgeConfiguration({ ...env, WA_BRIDGE_ADDR: '0.0.0.0:39819' }));
  assert.throws(() => bridgeConfiguration({ ...env, WA_BRIDGE_KEY_HEX: '00'.repeat(32) }));
  assert.throws(() => bridgeConfiguration({ WA_BRIDGE_ADDR: env.WA_BRIDGE_ADDR }));
  assert.throws(() => bridgeConfiguration({ ...env, WA_BRIDGE_OWNER_JID: 'group@g.us' }));
});

test('only own authenticated private JID and canonical global prefix are forwarded', () => {
  assert.deepEqual(ownerControlPayload(message(), CONFIG, 1794873600), {
    version: 1,
    socket_id: 'socket-1', actor_id: 'owner', owner_jid: OWNER,
    chat_jid: OWNER, message_id: 'msg-1', text: '=/>status',
    from_me: true, timestamp: 1794873600,
  });
  const outsider = message();
  outsider.key.fromMe = false;
  assert.equal(ownerControlPayload(outsider, CONFIG), null);
  const group = message();
  group.key.remoteJid = '123@g.us';
  assert.equal(ownerControlPayload(group, CONFIG), null);
  const participant = message();
  participant.key.participant = 'another@s.whatsapp.net';
  assert.equal(ownerControlPayload(participant, CONFIG), null);
  assert.equal(ownerControlPayload(message('#ping'), CONFIG), null);
  assert.equal(ownerControlPayload(message('=on'), CONFIG), null);
  assert.equal(ownerControlPayload(message('/>off'), CONFIG), null);
  assert.equal(ownerControlPayload(message('=/>on', ''), CONFIG), null);
});

test('fixed signature vector matches Base-Matter Rust HMAC-SHA256 verifier', () => {
  const unsigned = ownerControlPayload(message(), CONFIG, 1794873600);
  const signed = signControlFrame(unsigned, KEY);
  assert.equal(
    signed.signature,
    '747dc915a2734e90d1f2042cc42df7e360f848efcacda76baa8f27925b3124c7',
  );
  assert.notEqual(
    signControlFrame({ ...unsigned, socket_id: 'socket-2' }, KEY).signature,
    signed.signature,
  );
  assert.notEqual(
    signControlFrame({ ...unsigned, text: '=/>off' }, KEY).signature,
    signed.signature,
  );
});

test('extracts ephemeral text without authorizing media or unbounded wrappers', () => {
  assert.equal(privateControlText({
    ephemeralMessage: { message: { extendedTextMessage: { text: '=/>on' } } },
  }), '=/>on');
  assert.equal(privateControlText({ imageMessage: { caption: '=/>on' } }), null);
  assert.equal(privateControlText(null), null);
});

test('signed frames do not leak keys or store secrets in their JSON', () => {
  const signed = signControlFrame(
    ownerControlPayload(message(), CONFIG, 1794873600), KEY,
  );
  const encoded = JSON.stringify(signed);
  assert.ok(encoded.includes('=/>status'));
  assert.ok(!encoded.includes(KEY.toString('hex')));
  assert.ok(!encoded.includes('WA_BRIDGE_KEY_HEX'));
});
