import type { Json } from './types';
export interface RecordedAssetManifest {
  version: 'browser-recorded-asset-v1';
  path: string;
  bytes: number;
  sha256: string;
}
const MAX_BYTES = 16 * 1024 * 1024;
/** No request occurs until called; only the build-authenticated relative asset is accepted. */
export async function loadRecordedAsset(manifest: RecordedAssetManifest, base: URL, fetcher: typeof fetch = fetch): Promise<Json> {
  if (manifest.version !== 'browser-recorded-asset-v1' || !Number.isSafeInteger(manifest.bytes) || manifest.bytes < 1 || manifest.bytes > MAX_BYTES || !/^[a-f0-9]{64}$/.test(manifest.sha256) || manifest.path !== `recorded-assets/${manifest.sha256}.json`) throw new Error('Invalid recorded asset manifest');
  const response = await fetcher(new URL(manifest.path, base), { redirect: 'error' });
  if (!response.ok || !response.body) throw new Error(`Recorded asset unavailable (${response.status})`);
  const declared = response.headers.get('content-length');
  if (declared !== null && !response.headers.get('content-encoding') && (!/^\d+$/.test(declared) || Number(declared) !== manifest.bytes)) throw new Error('Recorded asset byte length mismatch');
  const reader = response.body.getReader();
  const bytes = new Uint8Array(manifest.bytes);
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > manifest.bytes || size > MAX_BYTES) throw new Error('Recorded asset exceeds byte limit');
      bytes.set(value, size - value.byteLength);
    }
  } catch (error) {
    await reader.cancel();
    throw error;
  } finally { reader.releaseLock(); }
  if (size !== manifest.bytes) throw new Error('Recorded asset byte length mismatch');
  const digest = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), b => b.toString(16).padStart(2, '0')).join('');
  if (digest !== manifest.sha256) throw new Error('Recorded asset SHA-256 mismatch');
  const value = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes)) as Json;
  if (!value || Array.isArray(value) || typeof value !== 'object' || value.kind !== 'recorded_results' || value.version !== 1 || !value.studies || typeof value.studies !== 'object' || Array.isArray(value.studies)) throw new Error('Invalid recorded asset envelope');
  return value;
}
