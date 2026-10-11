import { CachingControls } from './caching-controls';
import { SpatialControls } from './spatial-controls';
import { surfaceSettingLabel } from './comparison';
import { h } from '../ui/dom';
import { obj, list, label, valueText } from './presentation';
import type { Json, StudyDescriptor } from './types';
export function checkedSurfaceSequence(value: string): number {
  if (!/^[0-9]+$/.test(value) || Number(value) > 255) throw new Error('sequence: enter a whole number from 0 to 255');
  return Number(value);
}
export function surfaceSequenceValue(opened: Json, current: string, edited: boolean): Json {
  return edited || typeof opened === 'number' ? checkedSurfaceSequence(current) : opened;
}
/** These are the native family's named selectors, not a user-authored engine configuration. */
export class EpisodeControls {
  readonly el = h('div', { class: 'episode-controls' });
  private draft: Record<string, Json>;
  private sequence: HTMLInputElement | null = null;
  private sequenceEdited = false;
  private spatial: SpatialControls | null = null;
  constructor(private descriptor: StudyDescriptor, input: Json = descriptor.default_input, private changed: (input: Json) => void = () => {}) {
    this.draft = structuredClone(obj(input));
    if (descriptor.family === 'spatial') { this.spatial = new (['protection_recaching','deception_gestures'].includes(descriptor.id)?CachingControls:SpatialControls)(descriptor, input, changed); this.el.append(this.spatial.el); } else this.render();
  }
  input(): Json {
    if (this.spatial) return this.spatial.input();
    const input = structuredClone(this.draft);
    if (this.sequence && ('sequence' in input || this.sequenceEdited)) input.sequence = surfaceSequenceValue(input.sequence ?? null, this.sequence.value, this.sequenceEdited);
    return input;
  }
  private renderSurfaceControls(): void {
    const rows = list(obj(this.descriptor.controls).settings).map(obj);
    const selected = rows.findIndex(row => {
      const protocol = { ...obj(row.protocol), ids: obj(this.draft.protocol).ids };
      return JSON.stringify(protocol) === JSON.stringify(this.draft.protocol) && JSON.stringify(row.environment) === JSON.stringify(this.draft.environment);
    });
    const picker = h('select', { 'aria-label': 'Surface setting' }, ...rows.map((row, index) => h('option', { value: index, selected: index === selected }, `${index + 1}: ${surfaceSettingLabel(row)}`)));
    if (selected < 0) picker.prepend(h('option', { value: '', selected: true }, 'Opened input · validation on run'));
    picker.addEventListener('change', () => {
      const row = rows[Number(picker.value)];
      this.draft.environment = structuredClone(row.environment);
      this.draft.protocol = { ...structuredClone(obj(row.protocol)), ids: structuredClone(obj(this.draft.protocol).ids) };
      try { this.changed(this.input()); }
      catch { this.sequence?.setAttribute('aria-invalid', 'true'); }
    });
    const sequence = h('input', { type: 'number', min: 0, max: 255, step: 1, value: this.draft.sequence, 'aria-label': 'Complete bit sequence (0–255)' });
    this.sequence = sequence;
    sequence.addEventListener('input', () => {
      this.sequenceEdited = true;
      try { const input = this.input(); sequence.removeAttribute('aria-invalid'); this.changed(input); }
      catch { sequence.setAttribute('aria-invalid', 'true'); }
    });
    this.el.append(h('label', {}, this.descriptor.id === 'surface_discovery' ? 'Registered discovery setting' : 'Original surface setting', picker), h('label', {}, 'Complete bit sequence (0–255)', sequence), h('p', { class: 'hint' }, this.descriptor.id === 'surface_discovery' ? 'Primary policies: RawAdaptive, SuppliedAdaptive, NoProbe. KnownRaw and InspectOnly are controls. The informed helper, seven-symbol alphabet, prices, and live routines are supplied and fixed. Raw actions select owned paid calibration slots; round purchase is a free boundary choice. Each purchased round charges two credits per Agent.' : 'Ordered original settings. RestartUniform and Stale describe externally supplied priors; each episode starts fresh with blank fields.'));
  }
  private render(): void {
    this.el.replaceChildren();
    if (this.descriptor.family === 'surface') { this.renderSurfaceControls(); return; }
    const controls = obj(this.descriptor.controls);
    const keys = this.descriptor.id === 'wink' ? ['seed', 'policy', 'mode'] : this.descriptor.id === 'testimony' ? ['fixture'] : ['environment', 'listener', 'policy', 'controller', 'witness', 'catalog', 'history'];
    for (const key of keys) {
      if (!(key in controls)) continue;
      const spec = obj(controls[key]);
      let input: HTMLSelectElement | HTMLInputElement;
      if (key === 'history') {
        input = h('input', { type: 'number', min: spec.min, max: spec.max, step: 1, value: this.draft[key], 'aria-label': 'Public history' });
      } else if (key === 'seed') {
        input = h('input', { type: 'text', inputMode: 'numeric', pattern: '[0-9]+', value: this.draft[key], 'aria-label': 'Seed (decimal u64)' });
      } else {
        const perEnvironment = list(spec.by_environment).find(row => obj(row).environment === this.draft.environment);
        const values = list(perEnvironment ? obj(perEnvironment).values : spec.values);
        input = h('select', { 'aria-label': label(key) }, ...values.map(value => h('option', { value, selected: value === this.draft[key] }, obj(spec.labels)[String(value)] ? valueText(obj(spec.labels)[String(value)]) : label(String(value)))));
        if (!values.includes(this.draft[key])) {
          input.prepend(h('option', { value: '', selected: true, disabled: true }, `Opened ${label(key)}: ${key in this.draft ? valueText(this.draft[key]) : 'missing'} · validation on run`));
          input.setAttribute('aria-invalid', 'true');
        }
      }
      input.addEventListener('change', () => {
        this.draft[key] = key === 'history' ? Number(input.value) : input.value;
        if (key === 'environment') {
          // Only an intentional environment edit may reset its dependent choices.
          // Rendering an opened input must preserve every value for native validation.
          for (const [dependent, control] of Object.entries(controls)) {
            const row = list(obj(control).by_environment).find(row => obj(row).environment === this.draft.environment);
            const values = list(obj(row).values);
            if (row && values.length && !values.includes(this.draft[dependent])) this.draft[dependent] = values[0];
          }
          this.render();
        } else input.removeAttribute('aria-invalid');
        this.changed(this.input());
      });
      this.el.append(h('label', {}, key === 'history' ? 'Public history' : label(key), input));
    }
    if ('history' in controls) this.el.append(h('p', { class: 'hint' }, 'A conditional public history (0–31). This selects a case; private truth is not sampled.'));
  }
}
