/*
 * Credits: Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
import { test } from "node:test";
import { strict as assert } from "node:assert";
import { describeRequest, parseResponse } from "./index.mjs";

test("safe describe request", () => {
  assert.deepEqual(describeRequest("whatsapp-rust"), {
    version: 1, operation: "describe", provider: "whatsapp-rust",
  });
  assert.throws(() => describeRequest("../../vault"), TypeError);
});

test("rejects mutating error and unauthenticated descriptor", () => {
  assert.throws(() => parseResponse('{"version":1,"ok":false,"error":"operation_not_allowed"}'));
  assert.throws(() => parseResponse('{"version":1,"ok":true,"provider":{"name":"baileys","runtime":"Rust","control_authority":true}}'), TypeError);
});

test("reads descriptor", () => {
  const desc = parseResponse(JSON.stringify({
    version: 1, ok: true, provider: {
      name: "whatsmeow", runtime: "Rust", original_ecosystem: "Go",
      status: "compatibility_only", operations: ["describe"],
      control_authority: false,
    },
  }));
  assert.equal(desc.name, "whatsmeow");
});
