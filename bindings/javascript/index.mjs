/*
 * WhatsApp-Matter — JavaScript binding for READ-ONLY bridge metadata.
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
export const PROTOCOL_VERSION = 1;
const PROVIDERS = new Set(["whatsapp-rust", "baileys", "whatsmeow"]);

export function describeRequest(provider) {
  if (!PROVIDERS.has(provider)) {
    throw new TypeError("Unsupported provider");
  }
  return { version: PROTOCOL_VERSION, operation: "describe", provider };
}

export function parseResponse(line) {
  if (typeof line !== "string" || line.length > 8192) {
    throw new TypeError("Invalid bridge response");
  }
  const response = JSON.parse(line);
  if (typeof response !== "object" || response === null ||
      response.version !== PROTOCOL_VERSION || typeof response.ok !== "boolean") {
    throw new TypeError("Malformed bridge response");
  }
  if (!response.ok) {
    throw new Error(typeof response.error === "string" ? response.error : "bridge_denied");
  }
  if (!response.provider || !PROVIDERS.has(response.provider.name) ||
      response.provider.runtime !== "Rust" ||
      response.provider.control_authority !== false) {
    throw new TypeError("Untrusted provider descriptor");
  }
  return response.provider;
}
