// Test-only authentication of the exact reviewed incoming changes against
// immutable historical measurement receipts. This never changes engine identity.
import { readFileSync } from 'node:fs';
import { expect } from 'vitest';
const repository = new URL('../../../../', import.meta.url);
type Receipt = { path: string; fixture: string; historical_sha256: string; reviewed_sha256: string };
const compatibility = JSON.parse(new TextDecoder().decode(readFileSync(new URL('web/src/episodes/test-fixtures/foraging-source-compatibility.json', repository)))) as { sources: Receipt[] };
async function sha(bytes: Uint8Array<ArrayBuffer>): Promise<string> {
  return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), n => n.toString(16).padStart(2, '0')).join('');
}
export async function historicalSourceBytes(path: string, live: Uint8Array<ArrayBuffer>, expectedHistorical?: string): Promise<Uint8Array<ArrayBuffer>> {
  if (path !== 'crates/sugarscape-core/src/lib.rs') {
    const receipt = compatibility.sources.find(source => source.path === path);
    if (!receipt) return live;
    if (expectedHistorical !== undefined) expect(receipt.historical_sha256, `${path}: mapping must match the caller's frozen receipt`).toBe(expectedHistorical);
    expect(await sha(live), `${path}: reviewed incoming source bytes`).toBe(receipt.reviewed_sha256);
    const historical = readFileSync(new URL(receipt.fixture, repository));
    expect(await sha(historical), `${path}: original historical source bytes`).toBe(receipt.historical_sha256);
    return historical;
  }
  const historical = readFileSync(new URL('web/src/episodes/test-fixtures/pre-war-core-lib.rs.txt', repository));
  const original = '6ac710e9fe3521e6591dc8edc02b884e2aaae55a36298ab7313d9a9fb4c85f4f';
  if (expectedHistorical !== undefined) expect(expectedHistorical).toBe(original);
  expect(await sha(historical)).toBe(original);
  const reviewedAdditions = '\npub mod surface_discovery;\n\n#[cfg(all(feature = "war-benchmarks", not(target_arch = "wasm32")))]\npub mod war;\n\n/// Six-round paid gate assistance with local histories.\npub mod gate_assistance;\n\n/// Six-round carrier-only paid key delegation with local histories.\npub mod key_delegation;\n\n/// Assistance games: the paperclip game and the off-switch game (milestone 39).\npub mod assistance;\n\n/// AI Safety Gridworlds: eight environments with hidden performance functions (milestone 40).\npub mod gridworlds;\n\n/// Consequences of misaligned AI: a proxy on a subset of the attributes (milestone 43).\npub mod misaligned;\n';
  const additionBytes = new TextEncoder().encode(reviewedAdditions);
  const expectedLive = new Uint8Array(historical.length + additionBytes.length);
  expectedLive.set(historical); expectedLive.set(additionBytes, historical.length);
  expect(new Uint8Array(live), 'live lib.rs differs beyond the reviewed discovery/Gate/Key/assistance/gridworlds/misaligned exports and native W1 gate').toEqual(expectedLive);
  return historical;
}
