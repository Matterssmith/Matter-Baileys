/*
 * WhatsApp-Matter — offline message admission tests.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  parseAllowlist,
  messageText,
  classifyPrivateTestMessage,
  makeBoundedReplay,
} from '../src/guard.mjs';

const jid = '5215512345678@s.whatsapp.net';
function message(text = '#ping', id = 'event-1') {
  return { key: { remoteJid: jid, id, fromMe: false }, message: { conversation: text } };
}

test('empty allowlist denies every reply, including #ping', () => {
  assert.equal(classifyPrivateTestMessage(message(), new Set(), true), null);
});

test('read-only mode denies all outbound responses', () => {
  assert.equal(classifyPrivateTestMessage(message(), new Set([jid]), false), null);
});

test('only explicit private sender and exact ping are accepted', () => {
  const allowed = new Set([jid]);
  assert.deepEqual(classifyPrivateTestMessage(message(), allowed, true), {
    jid, eventId: 'event-1', reply: 'pong',
  });
  assert.equal(classifyPrivateTestMessage(message('#menu'), allowed, true), null);
  assert.equal(classifyPrivateTestMessage(message('hi'), allowed, true), null);
  assert.equal(classifyPrivateTestMessage(message('#ping', ''), allowed, true), null);
});

test('group, status, own messages and unknown contacts never reply', () => {
  const allowed = new Set([jid, '123@g.us']);
  const group = message();
  group.key.remoteJid = '123@g.us';
  assert.equal(classifyPrivateTestMessage(group, allowed, true), null);
  const fromMe = message();
  fromMe.key.fromMe = true;
  assert.equal(classifyPrivateTestMessage(fromMe, allowed, true), null);
  const wrongSender = message();
  wrongSender.key.remoteJid = '999@s.whatsapp.net';
  assert.equal(classifyPrivateTestMessage(wrongSender, allowed, true), null);
  const status = message();
  status.key.remoteJid = 'status@broadcast';
  assert.equal(classifyPrivateTestMessage(status, allowed, true), null);
});

test('ephemeral and extended text messages can be inspected safely', () => {
  const nested = { ephemeralMessage: { message: { extendedTextMessage: { text: '#ping' } } } };
  assert.equal(messageText(nested), '#ping');
  assert.equal(messageText(null), null);
});

test('allowlist accepts verified JIDs and rejects malformed or group addresses', () => {
  const allowed = parseAllowlist(' 5215512345678@s.whatsapp.net, example@lid ');
  assert.equal(allowed.size, 2);
  assert.ok(allowed.has(jid));
  assert.throws(() => parseAllowlist('123@g.us'));
  assert.throws(() => parseAllowlist('injected@localhost'));
  assert.throws(() => parseAllowlist('x'.repeat(140) + '@lid'));
});

test('bounded replay rejects duplicates and does not grow indefinitely', () => {
  const replay = makeBoundedReplay(2);
  assert.ok(replay.claim('a'));
  assert.equal(replay.claim('a'), false);
  assert.ok(replay.claim('b'));
  assert.ok(replay.claim('c'));
  assert.ok(replay.claim('a'));
  assert.throws(() => makeBoundedReplay(0));
});
