/*
 * WhatsApp-Matter — isolated, owner-controlled WhatsApp Web smoke test.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */

import { mkdir, open, unlink } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { dirname } from 'node:path';
import { classifyPrivateTestMessage, makeBoundedReplay, parseAllowlist } from './guard.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function configuration() {
  const authDir = resolve(process.env.WA_AUTH_DIR ?? join(root, '.local-auth'));
  if (authDir === root || !authDir.startsWith(root + '/')) {
    throw new Error('WA_AUTH_DIR must be a private directory under this checkout');
  }
  const pairingPhone = process.env.WA_PAIRING_PHONE?.trim() ?? '';
  if (pairingPhone && !/^[0-9]{8,16}$/.test(pairingPhone)) {
    throw new Error('WA_PAIRING_PHONE must contain only international phone digits');
  }
  return {
    authDir,
    pairingPhone,
    sendEnabled: process.env.WA_TEST_SEND === '1',
    allowedJids: parseAllowlist(process.env.WA_ALLOWED_JIDS ?? ''),
  };
}

async function main() {
  if (process.argv.includes('--help')) {
    process.stdout.write('WhatsApp-Matter smoke: npm run smoke. Requires an authorized secondary number.\n');
    return;
  }
  const config = configuration();
  await mkdir(config.authDir, { recursive: true, mode: 0o700 });
  const lockPath = join(config.authDir, '.owner.lock');
  let lock;
  try {
    lock = await open(lockPath, 'wx', 0o600);
    await lock.writeFile(String(process.pid));
  } catch {
    throw new Error('Session owned or lock stale; do not start a competing socket');
  }
  let released = false;
  async function releaseLock() {
    if (released) return;
    released = true;
    try { await lock.close(); } catch { /* cleanup only */ }
    try { await unlink(lockPath); } catch { /* cleanup only */ }
  }
  try {
    const [{ default: makeWASocket, useMultiFileAuthState }, { default: qrTerm }, { default: pino }] =
      await Promise.all([
        import('@whiskeysockets/baileys'),
        import('qrcode-terminal'),
        import('pino'),
      ]);
    const { state, saveCreds } = await useMultiFileAuthState(config.authDir);
    const socket = makeWASocket({
      auth: state,
      logger: pino({ level: 'silent' }),
      printQRInTerminal: false,
      syncFullHistory: false,
      markOnlineOnConnect: false,
    });
    const replay = makeBoundedReplay();
    let pairingRequested = false;
    socket.ev.on('creds.update', () => {
      void saveCreds().catch(() => {
        process.stderr.write('Credential persistence failed; stop this test session.\n');
        process.exitCode = 1;
      });
    });
    socket.ev.on('connection.update', (update) => {
      if (update.qr && !config.pairingPhone) {
        qrTerm.generate(update.qr, { small: true });
      }
      if (update.qr && config.pairingPhone && !state.creds.registered && !pairingRequested) {
        pairingRequested = true;
        void socket.requestPairingCode(config.pairingPhone)
          .then((code) => process.stdout.write('Temporary pairing code: ' + code + '\n'))
          .catch(() => process.stderr.write('Pairing code request failed; use QR or inspect provider setup.\n'));
      }
      if (update.connection === 'open') {
        process.stdout.write('WhatsApp test socket connected. Read-only by default.\n');
      }
      if (update.connection === 'close') {
        process.stderr.write('WhatsApp test socket disconnected. No automatic reconnect.\n');
        void releaseLock();
        process.exitCode = 1;
      }
    });
    socket.ev.on('messages.upsert', ({ messages, type }) => {
      if (type !== 'notify' || !Array.isArray(messages)) return;
      for (const message of messages) {
        const eligible = classifyPrivateTestMessage(message, config.allowedJids, config.sendEnabled);
        if (!eligible || !replay.claim(eligible.jid + ':' + eligible.eventId)) continue;
        void socket.sendMessage(eligible.jid, { text: eligible.reply })
          .then(() => process.stdout.write('Test ping acknowledged to an allowed private contact.\n'))
          .catch(() => process.stderr.write('Test reply failed; inspect connection state.\n'));
      }
    });
    for (const signal of ['SIGINT', 'SIGTERM']) {
      process.once(signal, () => {
        try { socket.end(new Error('Test stopped by operator')); } catch { /* best effort */ }
        void releaseLock();
      });
    }
    process.stdout.write(
      'Test socket initializing. ' +
      (config.sendEnabled && config.allowedJids.size ? 'Allowlisted ping reply enabled.\n' : 'Read-only; no replies.\n'),
    );
  } catch (error) {
    await releaseLock();
    throw error;
  }
}

main().catch((error) => {
  process.stderr.write('WhatsApp smoke startup failed: ' + error.message + '\n');
  process.exitCode = 1;
});
