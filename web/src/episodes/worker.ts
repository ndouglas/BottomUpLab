// This worker owns its WASM instance. The page terminates it on every end state.
import init, { experiment_catalog_json, experiment_run_json, experiment_validate_json } from '../experiments-pkg/experiments.js';
import { loadRecordedAsset, type RecordedAssetManifest } from './recorded-assets';
import manifest from './recorded-manifest.json';
import { parseErrors } from '../types';
import type { EpisodeRecord, StudyDescriptor, WorkerReply, WorkerRequest } from './types';
let started = false;
addEventListener('message', async ({ data }: MessageEvent<WorkerRequest>) => {
  if (started) {
    postMessage({ id: data.id, errors: [{ field: 'worker', message: 'worker already owns a request' }] } satisfies WorkerReply);
    return;
  }
  started = true;
  let reply: WorkerReply;
  try {
    // Initialization happens inside try, after a request: rejected loads are never unhandled.
    if (data.op === 'recorded') {
      const base = new URL(import.meta.env.DEV ? '../../../' : '../', import.meta.url);
      const recorded = await loadRecordedAsset(manifest as RecordedAssetManifest, base);
      postMessage({ id: data.id, kind: 'recorded', recorded } satisfies WorkerReply);
      return;
    }
    await init();
    switch (data.op) {
      case 'run':
        reply = { id: data.id, kind: 'episode', record: JSON.parse(experiment_run_json(data.text)) as EpisodeRecord };
        break;
      case 'validate':
        reply = { id: data.id, kind: 'episode', record: JSON.parse(experiment_validate_json(data.text)) as EpisodeRecord };
        break;
      case 'catalog':
        reply = { id: data.id, kind: 'catalog', catalog: JSON.parse(experiment_catalog_json()) as StudyDescriptor[] };
        break;
      default:
        throw new Error('unknown episode worker operation');
    }
  } catch (error) {
    reply = { id: data.id, errors: parseErrors(error, 'worker') };
  }
  postMessage(reply);
});
