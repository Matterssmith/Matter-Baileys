/*
 * WhatsApp-Matter — TypeScript types for the read-only Rust contract.
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
export type ProviderName = "whatsapp-rust" | "baileys" | "whatsmeow";
export type ProviderStatus = "native_rust_migration" | "compatibility_only";

export interface DescribeRequest {
  version: 1;
  operation: "describe";
  provider: ProviderName;
}

export interface ProviderDescriptor {
  name: ProviderName;
  runtime: "Rust";
  original_ecosystem: string;
  status: ProviderStatus;
  operations: readonly string[];
  control_authority: false;
}

export interface DescribeResponse {
  version: 1;
  ok: boolean;
  provider?: ProviderDescriptor;
  error?: string;
}

export function describeRequest(provider: ProviderName): DescribeRequest {
  return { version: 1, operation: "describe", provider };
}

export function parseDescribeResponse(value: string): ProviderDescriptor {
  if (value.length > 8192) throw new Error("response_too_large");
  const data: unknown = JSON.parse(value);
  if (typeof data !== "object" || data === null) throw new Error("invalid_response");
  const response = data as Partial<DescribeResponse>;
  if (response.version !== 1 || response.ok !== true || !response.provider) {
    throw new Error(response.error ?? "bridge_denied");
  }
  const provider = response.provider;
  if (provider.runtime !== "Rust" || provider.control_authority !== false ||
      !["whatsapp-rust", "baileys", "whatsmeow"].includes(provider.name)) {
    throw new Error("untrusted_descriptor");
  }
  return provider;
}
