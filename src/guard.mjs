/*
 * WhatsApp-Matter — code-only pairing admission.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */

export function validInternationalPhone(phone) {
  return typeof phone === 'string'
    && /^[0-9]{8,15}$/.test(phone);
}

export function pairingCodeValid(code) {
  return typeof code === 'string'
    && /^[A-Z0-9]{6,16}$/.test(code);
}

/**
 * Baileys may report an internal QR challenge even during pairing-code
 * login. Only treat that as a transport-readiness signal; NEVER display,
 * export, store or otherwise use a QR as a login option.
 */
export function shouldRequestPairingCode(update, registered, requested) {
  return Boolean(
    update
    && !registered
    && !requested
    && (update.connection === 'connecting' || typeof update.qr === 'string')
  );
}
