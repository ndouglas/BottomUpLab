import { NotesControls } from './notes-controls';
import { KeyControls } from './key-controls';
import { gateSelectorLabels } from './gate-comparison';
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
export function checkedGateId(value: string, role: 'holder' | 'retriever'): number {
  const max = role === 'holder' ? 728 : 63;
  if (!/^[0-9]+$/.test(value) || Number(value) > max) throw new Error(`${role}: enter a whole number from 0 to ${max}`);
  return Number(value);
}
export function gateNumericValue(opened: Json, current: string, edited: boolean, role: 'holder' | 'retriever'): Json {
  return edited || typeof opened === 'number' ? checkedGateId(current, role) : opened;
}
/** These are the native family's named selectors, not a user-authored engine configuration. */
export class EpisodeControls {
  readonly el = h('div', { class: 'episode-controls' });
  private draft: Record<string, Json>;
  private sequence: HTMLInputElement | null = null;
  private sequenceEdited = false;
  private gateFields: { role: 'holder' | 'retriever'; el: HTMLInputElement; edited: boolean }[] = [];
  private gateIds: { role: string; el: HTMLInputElement; edited: boolean }[] = [];
  private key: KeyControls | NotesControls | null = null;
  private spatial: SpatialControls | null = null;
  constructor(private descriptor: StudyDescriptor, input: Json = descriptor.default_input, private changed: (input: Json) => void = () => {}) {
    this.draft = structuredClone(obj(input));
    if (descriptor.family === 'notes') { this.key = new NotesControls(descriptor, input, changed); this.el.append(this.key.el); } else if (descriptor.family === 'key') { this.key = new KeyControls(descriptor, input, changed); this.el.append(this.key.el); } else if (descriptor.family === 'spatial') { this.spatial = new (['protection_recaching','deception_gestures'].includes(descriptor.id)?CachingControls:SpatialControls)(descriptor, input, changed); this.el.append(this.spatial.el); } else this.render();
  }
  input(): Json {
    if (this.key) return this.key.input();
    if (this.spatial) return this.spatial.input();
    const input = structuredClone(this.draft);
    if (this.descriptor.family === 'gate') {
      for (const field of this.gateFields) {
        const profile = obj(input.profile);
        if (field.role in profile || field.edited) profile[field.role] = gateNumericValue(profile[field.role] ?? null, field.el.value, field.edited, field.role);
      }
      for (const field of this.gateIds) if (field.role in obj(input.ids) || field.edited) obj(input.ids)[field.role] = field.edited || typeof obj(input.ids)[field.role] === 'string' ? field.el.value : obj(input.ids)[field.role];
      return input;
    }
    if (this.sequence && ('sequence' in input || this.sequenceEdited)) input.sequence = surfaceSequenceValue(input.sequence ?? null, this.sequence.value, this.sequenceEdited);
    return input;
  }
  private renderGateControls(): void {
    this.gateFields = []; this.gateIds = [];
    this.el.append(h('p', { class: 'hint' }, 'Researcher controls for the next episode. Complete schedule selection is outside each Agent view.'));
    for (const key of ['holding_cost', 'holder_share', 'distance']) {
      const values = list(obj(this.descriptor.controls)[key]), selected = obj(this.draft.stratum)[key];
      const picker = h('select', { 'aria-label': `Gate ${label(key)}` }, ...values.map(value => h('option', { value, selected: value === selected }, valueText(value))));
      if (!values.includes(selected)) picker.prepend(h('option', { value: '', selected: true }, 'Opened value · validation on run'));
      picker.addEventListener('change', () => { obj(this.draft.stratum)[key] = Number(picker.value); this.changed(this.input()); });
      this.el.append(h('label', {}, label(key), picker));
    }
    const selected = 'profile' in this.draft ? 'raw' : this.draft.selector;
    const picker = h('select', { 'aria-label': 'Gate schedule selection' }, ...list(obj(this.descriptor.controls).selectors).map(selector => h('option', { value: selector, selected: selector === selected }, gateSelectorLabels[String(selector)] ?? valueText(selector))), h('option', { value: 'raw', selected: selected === 'raw' }, 'Fixed raw schedule pair'));
    if (selected !== 'raw' && !list(obj(this.descriptor.controls).selectors).includes(selected)) picker.prepend(h('option', { value: '', selected: true }, 'Opened choice · validation on run'));
    picker.addEventListener('change', () => {
      if (picker.value === 'raw') { delete this.draft.selector; this.draft.profile = { holder: 0, retriever: 0 }; }
      else { delete this.draft.profile; this.draft.selector = picker.value; }
      this.render(); this.changed(this.input());
    });
    this.el.append(h('label', {}, 'Schedule selection', picker));
    if ('profile' in this.draft) for (const role of ['holder', 'retriever'] as const) {
      const el = h('input', { type: 'text', inputMode: 'numeric', value: obj(this.draft.profile)[role] ?? '', 'aria-label': `Raw ${role} word ID` });
      const field = { role, el, edited: false }; this.gateFields.push(field);
      el.addEventListener('input', () => { field.edited = true; try { this.changed(this.input()); el.removeAttribute('aria-invalid'); } catch { el.setAttribute('aria-invalid', 'true'); } });
      this.el.append(h('label', {}, `${label(role)} word ID (0–${role === 'holder' ? 728 : 63})`, el));
    }
    for (const role of ['holder', 'retriever']) {
      const el = h('input', { type: 'text', value: obj(this.draft.ids)[role] ?? '', 'aria-label': `Gate ${role} Agent ID` });
      const field = { role, el, edited: false }; this.gateIds.push(field);
      el.addEventListener('input', () => { field.edited = true; this.draft.ids = { ...obj(this.draft.ids), [role]: el.value }; try { this.changed(this.input()); } catch { el.setAttribute('aria-invalid', 'true'); } });
      this.el.append(h('label', {}, `${label(role)} Agent ID`, el));
    }
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
    if (this.descriptor.family === 'gate') { this.renderGateControls(); return; }
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
