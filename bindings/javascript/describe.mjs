/*
 * WhatsApp-Matter — End-to-end read-only Rust bridge example for JavaScript.
 * No WhatsApp pairing, session data or privileged command is exposed.
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
import { spawnSync } from "node:child_process";
import { describeRequest, parseResponse } from "./index.mjs";

const provider = process.argv[2] ?? "whatsapp-rust";
const payload = JSON.stringify(describeRequest(provider)) + "\n";
const proc = spawnSync("cargo", [
  "run", "--quiet", "-p", "matter-whatsapp-contracts",
  "--bin", "matter_whatsapp_bridge",
], { input: payload, encoding: "utf8", timeout: 120000, maxBuffer: 65536 });
if (proc.error || proc.status !== 0) {
  console.error(proc.error?.message ?? "Rust bridge unavailable");
  process.exitCode = 1;
} else {
  const output = proc.stdout.trim().split("\n")[0] ?? "";
  try {
    console.log(JSON.stringify(parseResponse(output), null, 2));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
