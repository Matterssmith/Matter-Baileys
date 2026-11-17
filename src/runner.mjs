/*
 * WhatsApp-Matter — owner-controlled code-only connection probe.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */

import { mkdir, open, unlink } from 'node:fs/promises';
import { resolve, join, relative, isAbsolute, dirname, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { pairingCodeValid, shouldRequestPairingCode, validInternationalPhone } from './guard.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function configuration() {
  const authDir = resolve(process.env.WA_AUTH_DIR ?? join(root, '.local-auth'));
  const location = relative(root, authDir);
  if (!location || location === '..' || location.startsWith('..' + sep) || isAbsolute(location)) {
    throw new Error('WA_AUTH_DIR must be a private directory under the checkout');
  }
  const phone = (process.env.WA_PAIRING_PHONE ?? '').trim();
  if (phone && !validInternationalPhone(phone)) {
    throw new Error('WA_PAIRING_PHONE must contain 8-15 international digits');
  }
  return { authDir, phone };
}

async function main() {
  if (process.argv.includes('--help')) {
    process.stdout.write(
      'WhatsApp-Matter pairing-code-only probe. No QR, no bot commands and no message sending.\n'
    );
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
    throw new Error('Session already owned or lock remains; never start a competing process');
  }

  let released = false;
  async function release() {
    if (released) return;
    released = true;
    try { await lock.close(); } catch { /* cleanup only */ }
    try { await unlink(lockPath); } catch { /* cleanup only */ }
  }

  try {
    const [{ default: makeWASocket, useMultiFileAuthState }, { default: pino }] =
      await Promise.all([import('@whiskeysockets/baileys'), import('pino')]);
    const { state, saveCreds } = await useMultiFileAuthState(config.authDir);
    if (!state.creds.registered && !config.phone) {
      throw new Error('WA_PAIRING_PHONE is required for an unlinked session; QR is not supported');
    }
    const socket = makeWASocket({
      auth: state,
      logger: pino({ level: 'silent' }),
      syncFullHistory: false,
      markOnlineOnConnect: false,
    });

    let pairingRequested = false;
    socket.ev.on('creds.update', () => {
      void saveCreds().catch(() => {
        process.stderr.write('Auth state write failed. Stop the test connection.\n');
        process.exitCode = 1;
      });
    });
    socket.ev.on('connection.update', (update) => {
      // An internal readiness challenge must never become a QR login option.
      if (shouldRequestPairingCode(update, state.creds.registered, pairingRequested)) {
        pairingRequested = true;
        void socket.requestPairingCode(config.phone)
          .then((code) => {
            if (!pairingCodeValid(code)) {
              process.stderr.write('Provider returned an invalid code.\n');
              return;
            }
            process.stdout.write('Pairing code for the authorized owner: ' + code + '\n');
          })
          .catch(() => process.stderr.write('Pairing code request failed; inspect provider setup.\n'));
      }
      if (update.connection === 'open') {
        process.stdout.write('Code-linked test connection open. Outbound sends disabled.\n');
      }
      if (update.connection === 'close') {
        process.stderr.write('Connection closed. No automatic reconnect or session deletion.\n');
        void release();
        process.exitCode = 1;
      }
    });

    // No messages.upsert listener, no sendMessage and no ping handler.
    for (const signal of ['SIGINT', 'SIGTERM']) {
      process.once(signal, () => {
        try { socket.end(new Error('Operator stopped the test')); } catch { /* best effort */ }
        void release();
      });
    }
    process.stdout.write('Pairing-code-only test. No QR and no automatic replies.\n');
  } catch (error) {
    await release();
    throw error;
  }
}

main().catch((error) => {
  process.stderr.write('Pairing-code-only startup rejected: ' + error.message + '\n');
  process.exitCode = 1;
});
