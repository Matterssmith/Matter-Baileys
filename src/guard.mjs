/*
 * WhatsApp-Matter — strict test-only incoming message gate.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */

const PRIVATE_SUFFIXES = ['@s.whatsapp.net', '@lid'];

export function parseAllowlist(raw = '') {
  const items = raw.split(',').map((value) => value.trim()).filter(Boolean);
  if (items.length > 32) throw new Error('Too many allowed test recipients');
  const result = new Set();
  for (const item of items) {
    if (item.length > 128 || !PRIVATE_SUFFIXES.some((suffix) => item.endsWith(suffix))) {
      throw new Error('Only explicitly allowed private JIDs are accepted');
    }
    if (!/^[a-zA-Z0-9._:-]+@(?:s\.whatsapp\.net|lid)$/.test(item)) {
      throw new Error('Invalid allowed recipient');
    }
    result.add(item);
  }
  return result;
}

export function messageText(envelope) {
  let current = envelope;
  for (let depth = 0; depth < 3; depth++) {
    if (!current || typeof current !== 'object') return null;
    if (typeof current.conversation === 'string') return current.conversation;
    if (typeof current.extendedTextMessage?.text === 'string') {
      return current.extendedTextMessage.text;
    }
    current = current.ephemeralMessage?.message
      ?? current.viewOnceMessage?.message
      ?? current.viewOnceMessageV2?.message;
  }
  return null;
}

export function classifyPrivateTestMessage(message, allowedJids, sendEnabled) {
  if (!message || message.key?.fromMe || !message.message) return null;
  const jid = message.key?.remoteJid;
  if (typeof jid !== 'string' || jid.length > 128) return null;
  if (!PRIVATE_SUFFIXES.some((suffix) => jid.endsWith(suffix))) return null;
  if (!sendEnabled || !allowedJids.has(jid)) return null;
  const id = message.key?.id;
  if (typeof id !== 'string' || !id || id.length > 256) return null;
  const text = messageText(message.message);
  if (text?.trim().toLowerCase() !== '#ping') return null;
  return { jid, eventId: id, reply: 'pong' };
}

export function makeBoundedReplay(limit = 2048) {
  if (!Number.isInteger(limit) || limit <= 0 || limit > 100000) {
    throw new Error('Invalid replay limit');
  }
  const observed = new Set();
  return {
    claim(key) {
      if (typeof key !== 'string' || key.length > 512 || !key) return false;
      if (observed.has(key)) return false;
      observed.add(key);
      if (observed.size > limit) observed.delete(observed.values().next().value);
      return true;
    },
  };
}
