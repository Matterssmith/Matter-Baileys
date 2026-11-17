/*
 * WhatsApp-Matter — verify installed public upstream entry points offline.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import makeWASocket, { useMultiFileAuthState } from '@whiskeysockets/baileys';

test('pinned Baileys provides a WebSocket factory and auth-state helper', () => {
  assert.equal(typeof makeWASocket, 'function');
  assert.equal(typeof useMultiFileAuthState, 'function');
});
