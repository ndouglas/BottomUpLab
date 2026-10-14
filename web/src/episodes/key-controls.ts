import { h } from '../ui/dom';
import { obj, list, label, valueText } from './presentation';
import { keySelectorLabels } from './key-comparison';
import type { Json, StudyDescriptor } from './types';
export function checkedKeyId(text: string, role: 'giver' | 'receiver'): number {
  const max = role === 'giver' ? 728 : 63;
  if (!/^(0|[1-9][0-9]*)$/.test(text) || Number(text) > max) throw new Error(`${role}: enter a canonical decimal integer from 0 to ${max}`);
  return Number(text);
}
/** Opened invalid types remain invalid until edited; each read validates current text. */
export class KeyControls {
  readonly el = h('div', { class: 'key-controls' });
  private draft: Record<string, Json>;
  private fields: { role: 'giver' | 'receiver'; el: HTMLInputElement; edited: boolean }[] = [];
  private ids: { role: string; el: HTMLInputElement; edited: boolean }[] = [];
  constructor(private descriptor: StudyDescriptor, input: Json, private changed: (input: Json) => void) {
    this.draft = structuredClone(obj(input)); this.render();
  }
  input(): Json {
    const result = structuredClone(this.draft);
    for (const field of this.fields) {
      const opened = obj(result.profile)[field.role];
      if (field.edited || typeof opened === 'number') result.profile = { ...obj(result.profile), [field.role]: checkedKeyId(field.el.value, field.role) };
    }
    for (const field of this.ids) if (field.edited || typeof obj(result.ids)[field.role] === 'string') result.ids = { ...obj(result.ids), [field.role]: field.el.value };
    return result;
  }
  private notify(): void {
    try { this.changed(this.input()); this.el.removeAttribute('aria-invalid'); }
    catch { this.el.setAttribute('aria-invalid', 'true'); }
  }
  private render(): void {
    this.el.replaceChildren(); this.fields = []; this.ids = [];
    this.el.append(h('p', { class: 'hint' }, 'Researcher controls for the next episode. Complete schedules stay outside peer Agent views. Pass attempts cost credits even when transfer is disabled.'));
    for (const key of ['transfer_enabled', 'transfer_cost', 'giver_share', 'distance']) {
      const values = list(obj(this.descriptor.controls)[key]), selected = obj(this.draft.stratum)[key];
      const picker = h('select', { 'aria-label': `Key ${label(key)}` }, ...values.map(value => h('option', { value: String(value), selected: value === selected }, key === 'transfer_enabled' ? value ? 'Enabled' : 'Disabled' : valueText(value))));
      if (!values.includes(selected)) picker.prepend(h('option', { value: '', selected: true, disabled: true }, 'Opened value · validation on run'));
      picker.addEventListener('change', () => { const value = values.find(v => String(v) === picker.value); if (value !== undefined) this.draft.stratum = { ...obj(this.draft.stratum), [key]: value }; this.notify(); });
      this.el.append(h('label', {}, label(key), picker));
    }
    const selected = 'profile' in this.draft ? 'raw' : this.draft.selector;
    const choices = list(obj(this.descriptor.controls).selectors);
    const picker = h('select', { 'aria-label': 'Key schedule selection' }, ...choices.map(value => h('option', { value, selected: value === selected }, keySelectorLabels[String(value)] ?? valueText(value))), h('option', { value: 'raw', selected: selected === 'raw' }, 'Fixed raw schedule pair'));
    if (selected !== 'raw' && !choices.includes(selected)) picker.prepend(h('option', { value: '', selected: true, disabled: true }, 'Opened choice · validation on run'));
    picker.addEventListener('change', () => {
      // Preserve current opaque names and raw text before replacing controls.
      try { this.draft = obj(this.input()); } catch { for (const field of this.ids) if (field.edited) this.draft.ids = { ...obj(this.draft.ids), [field.role]: field.el.value }; }
      if (picker.value === 'raw') { delete this.draft.selector; this.draft.profile = { giver: 0, receiver: 0 }; }
      else { delete this.draft.profile; this.draft.selector = picker.value; }
      this.render(); this.notify();
    });
    this.el.append(h('label', {}, 'Schedule selection', picker));
    if (selected === 'InitiallyEquipped') this.el.append(h('p', { class: 'hint' }, 'Changed initial ownership: Receiver starts with the key. Main-game regrets and equilibrium classification are unavailable.'));
    if ('profile' in this.draft) for (const role of ['giver', 'receiver'] as const) {
      const el = h('input', { type: 'text', inputMode: 'numeric', value: obj(this.draft.profile)[role] ?? '', 'aria-label': `Raw ${role} word ID` });
      const field = { role, el, edited: false }; this.fields.push(field);
      el.addEventListener('input', () => { field.edited = true; this.notify(); });
      this.el.append(h('label', {}, `${label(role)} word ID (0–${role === 'giver' ? 728 : 63})`, el));
    }
    for (const role of ['giver', 'receiver']) {
      const el = h('input', { type: 'text', value: obj(this.draft.ids)[role] ?? '', 'aria-label': `Key ${role} Agent ID` });
      const field = { role, el, edited: false }; this.ids.push(field);
      el.addEventListener('input', () => { field.edited = true; this.notify(); });
      this.el.append(h('label', {}, `${label(role)} Agent ID`, el));
    }
  }
}
