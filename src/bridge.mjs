/*
 * WhatsApp-Matter — signed owner-only control events for Base-Matter.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */

import { createHmac } from 'node:crypto';
import net from 'node:net';

const DOMAIN = Buffer.from('Base-Matter:WhatsApp-Control:v1\0', 'utf8');
const PRIVATE_JID = /^(?:[A-Za-z0-9._:-]+)@(?:s\.whatsapp\.net|lid)$/;
const ID = /^[A-Za-z0-9._:-]{1,128}$/;

export function bridgeConfiguration(env = process.env) {
  const names = [
    'WA_BRIDGE_ADDR', 'WA_BRIDGE_KEY_HEX', 'WA_BRIDGE_SOCKET_ID',
    'WA_BRIDGE_OWNER_ID', 'WA_BRIDGE_OWNER_JID',
  ];
  const present = names.filter((name) => !!env[name]?.trim());
  if (present.length === 0) return null;
  if (present.length !== names.length) {
    throw new Error('All WhatsApp control bridge settings are required');
  }
  const address = /^127\.0\.0\.1:(\d{1,5})$/.exec(env.WA_BRIDGE_ADDR.trim());
  const port = Number(address?.[1]);
  if (!address || port < 1 || port > 65535) {
    throw new Error('Bridge must use a loopback IPv4 port');
  }
  const keyHex = env.WA_BRIDGE_KEY_HEX.trim();
  if (!/^[a-fA-F0-9]{64}$/.test(keyHex) || /^0+$/.test(keyHex)) {
    throw new Error('Bridge needs a nonzero 256-bit key');
  }
  const socketId = env.WA_BRIDGE_SOCKET_ID.trim();
  const ownerActorId = env.WA_BRIDGE_OWNER_ID.trim();
  const ownerJid = env.WA_BRIDGE_OWNER_JID.trim();
  if (!ID.test(socketId) || !ID.test(ownerActorId) || !PRIVATE_JID.test(ownerJid)) {
    throw new Error('Invalid authenticated owner or socket ID');
  }
  return {
    port, socketId, ownerActorId, ownerJid,
    key: Buffer.from(keyHex, 'hex'),
  };
}

export function privateControlText(message) {
  let cursor = message;
  for (let depth = 0; depth < 3; depth++) {
    if (!cursor || typeof cursor !== 'object') return null;
    if (typeof cursor.conversation === 'string') return cursor.conversation;
    if (typeof cursor.extendedTextMessage?.text === 'string') {
      return cursor.extendedTextMessage.text;
    }
    cursor = cursor.ephemeralMessage?.message
      ?? cursor.viewOnceMessage?.message
      ?? cursor.viewOnceMessageV2?.message;
  }
  return null;
}

export function ownerControlPayload(message, config, timestamp = Math.floor(Date.now() / 1000)) {
  if (!config || !message?.key || !message?.message) return null;
  const { key } = message;
  if (key.fromMe !== true || key.remoteJid !== config.ownerJid || key.participant) {
    return null;
  }
  const messageId = key.id;
  if (typeof messageId !== 'string' || !messageId || messageId.length > 256
    || /[\x00-\x1f\x7f]/.test(messageId)) return null;
  const text = privateControlText(message.message);
  if (typeof text !== 'string' || !text.startsWith('=/>')
    || Buffer.byteLength(text, 'utf8') > 4096) return null;
  if (!Number.isSafeInteger(timestamp)) return null;
  return {
    version: 1,
    socket_id: config.socketId,
    actor_id: config.ownerActorId,
    owner_jid: config.ownerJid,
    chat_jid: config.ownerJid,
    message_id: messageId,
    text,
    from_me: true,
    timestamp,
  };
}

export function signControlFrame(unsigned, secret) {
  if (!Buffer.isBuffer(secret) || secret.length !== 32) {
    throw new Error('Invalid bridge key');
  }
  const mac = createHmac('sha256', secret);
  mac.update(DOMAIN);
  for (const value of [
    String(unsigned.version), unsigned.socket_id, unsigned.actor_id,
    unsigned.owner_jid, unsigned.chat_jid, unsigned.message_id, unsigned.text,
  ]) {
    if (typeof value !== 'string') throw new Error('Invalid bridge frame field');
    const bytes = Buffer.from(value, 'utf8');
    const length = Buffer.allocUnsafe(4);
    length.writeUInt32BE(bytes.length);
    mac.update(length);
    mac.update(bytes);
  }
  mac.update(Buffer.from([unsigned.from_me === true ? 1 : 0]));
  const timestamp = Buffer.allocUnsafe(8);
  timestamp.writeBigInt64BE(BigInt(unsigned.timestamp));
  mac.update(timestamp);
  return { ...unsigned, signature: mac.digest('hex') };
}

export async function deliverControlFrame(frame, config, timeoutMs = 3500) {
  if (!config || !Number.isInteger(timeoutMs) || timeoutMs < 100 || timeoutMs > 10000) {
    throw new Error('Invalid control bridge transport');
  }
  const json = JSON.stringify(frame) + '\n';
  if (Buffer.byteLength(json, 'utf8') > 16384) throw new Error('Control frame too large');
  return new Promise((resolve, reject) => {
    const connection = net.connect({ host: '127.0.0.1', port: config.port });
    let response = '';
    let settled = false;
    const finish = (err, value) => {
      if (settled) return;
      settled = true;
      connection.destroy();
      if (err) reject(new Error('Local Base-Matter control bridge unavailable'));
      else resolve(value);
    };
    connection.setTimeout(timeoutMs, () => finish(new Error('timeout')));
    connection.on('error', (error) => finish(error));
    connection.on('connect', () => connection.end(json));
    connection.on('data', (chunk) => {
      response += chunk.toString('utf8');
      if (response.length > 1024) return finish(new Error('oversized reply'));
      const newline = response.indexOf('\n');
      if (newline !== -1) {
        try {
          const data = JSON.parse(response.slice(0, newline));
          if (data.ok === true && typeof data.outcome === 'string') {
            finish(null, data.outcome);
          } else {
            finish(new Error('rejected'));
          }
        } catch {
          finish(new Error('bad reply'));
        }
      }
    });
    connection.on('end', () => finish(new Error('no reply')));
  });
}
