/*
 * WhatsApp-Matter — offline tests for code-only admission.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { validInternationalPhone, pairingCodeValid, shouldRequestPairingCode } from '../src/guard.mjs';

test('international digits are mandatory; plus signs and formatting rejected', () => {
  assert.equal(validInternationalPhone('5215512345678'), true);
  for (const candidate of ['+5215512345678', '52 15512345678', '', '12345', '1'.repeat(16)]) {
    assert.equal(validInternationalPhone(candidate), false);
  }
});

test('pairing provider codes must be bounded and alphanumeric', () => {
  assert.equal(pairingCodeValid('AB12CD34'), true);
  for (const candidate of ['', 'abc-defg', 'ABC', '12345678901234567']) {
    assert.equal(pairingCodeValid(candidate), false);
  }
});

test('provider readiness is checked only once on unregistered sessions', () => {
  assert.equal(shouldRequestPairingCode({ connection: 'connecting' }, false, false), true);
  assert.equal(shouldRequestPairingCode({ qr: 'INTERNAL_EVENT' }, false, false), true);
  assert.equal(shouldRequestPairingCode({ connection: 'connecting' }, true, false), false);
  assert.equal(shouldRequestPairingCode({ connection: 'connecting' }, false, true), false);
  assert.equal(shouldRequestPairingCode({}, false, false), false);
});

test('runner contains no rendered QR, outbound message send or ping handler', () => {
  const source = readFileSync(new URL('../src/runner.mjs', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /qrcode-terminal|qrTerm|\.sendMessage\(/);
  assert.doesNotMatch(source, /messages\.upsert|classifyPrivateTestMessage/);
  assert.match(source, /requestPairingCode\(config\.phone\)/);
});
