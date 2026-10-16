import { expect, it, vi } from 'vitest';
import { loadRecordedAsset, type RecordedAssetManifest } from './recorded-assets';
const bytes = new TextEncoder().encode('{"kind":"recorded_results","version":1,"studies":{}}');
const sha256 = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), b=>b.toString(16).padStart(2,'0')).join('');
const manifest: RecordedAssetManifest = { version: 'browser-recorded-asset-v1', path: `recorded-assets/${sha256}.json`, bytes: bytes.length, sha256 };
it('loads only on demand and verifies actual bytes', async () => {
  const fetcher = vi.fn(async () => new Response(bytes));
  expect(fetcher).not.toHaveBeenCalled();
  expect(await loadRecordedAsset(manifest, new URL('https://example.test/app/'), fetcher)).toEqual(JSON.parse(new TextDecoder().decode(bytes)));
  expect(fetcher).toHaveBeenCalledTimes(1);
});
it.each([{path:'../outside.json'}, {path:'https://evil.test/data'}, {path:'recorded-assets/%2e%2e/data.json'}, {version:'wrong'}, {bytes:16*1024*1024+1}, {bytes:-1}, {bytes:1.5}, {sha256:'x'.repeat(64)}])('rejects invalid manifest without network %j', async change => {
  const fetcher = vi.fn();
  await expect(loadRecordedAsset({...manifest,...change} as RecordedAssetManifest,new URL('https://example.test/'),fetcher)).rejects.toThrow();
  expect(fetcher).not.toHaveBeenCalled();
});
it('rejects corrupt, truncated, oversized streamed, missing and invalid-envelope assets', async () => {
  for (const response of [new Response('bad'),new Response(bytes.slice(1)),new Response(new Uint8Array(bytes.length+1)),new Response(null,{status:404})]) {
    await expect(loadRecordedAsset(manifest,new URL('https://example.test/'),async()=>response)).rejects.toThrow();
  }
  const bad = new TextEncoder().encode('{"kind":"recorded_results","version":2,"studies":{}}');
  const digest = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bad)), b=>b.toString(16).padStart(2,'0')).join('');
  await expect(loadRecordedAsset({...manifest,path:`recorded-assets/${digest}.json`,sha256:digest,bytes:bad.length},new URL('https://example.test/'),async()=>new Response(bad))).rejects.toThrow('envelope');
});

it('verifies decompressed stream bytes when transport has a compressed content length', async () => {
  const response = new Response(bytes, {headers:{'content-encoding':'gzip','content-length':'12'}});
  await expect(loadRecordedAsset(manifest,new URL('https://example.test/'),async()=>response)).resolves.toEqual(JSON.parse(new TextDecoder().decode(bytes)));
});
it('rejects same-length byte tampering by digest before parsing', async () => {
  const corrupted = bytes.slice(); corrupted[10] ^= 1;
  await expect(loadRecordedAsset(manifest,new URL('https://example.test/'),async()=>new Response(corrupted))).rejects.toThrow('SHA-256');
});
