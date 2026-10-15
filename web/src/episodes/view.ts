import { renderNotes } from './notes-view';
import { embeddedNotesAssets, notesComicSvg, notesComicHtml } from './notes-comic';
import { notesComparisonKey, notesMatchedInputs, matchNotesCheckpoint, type NotesAxis } from './notes-comparison';
import { renderKey, renderKeyResult } from './key-view';
import { keyComparisonKey, keyMatchedInputs, keySettingLabel, matchKeyCheckpoint } from './key-comparison';
import { h } from '../ui/dom';
import { downloadText } from '../downloads';
import { createEpisodeClient } from './client';
import { EpisodeSession, readEpisodeFile } from './file';
import { EpisodeControls } from './controls';
import { rendererAvailable, renderDescriptor, spatialCatalog } from './catalog';
import { ReplayController } from './replay';
import { projectCheckpoint, resultPayload, type Perspective } from './projection';
import { renderGate, renderGateResult } from './gate-view';
import { gateComparisonKey, gateMatchedInputs, gateSettingLabel } from './gate-comparison';
import { renderGame } from './game-view';
import { renderTestimony } from './testimony-view';
import { surfaceSummary } from './surface-projection';
import { renderSurface, renderSurfaceResult } from './surface-view';
import { comparisonKey, matchedInputs, matchingCheckpoint, surfaceSettingLabel } from './comparison';
import { renderCaching } from './caching-view';
import { renderSpatial, renderSpatialResult } from './spatial-view';
import { declaredSpatialAxis, spatialMatchedInputs, spatialComparisonKey, spatialAxisValue } from './spatial-comparison';
import { renderRecorded } from './recorded';
import { encodeEpisode, checkedInput } from './share';
import { obj, details, fact, label } from './presentation';
import type { EpisodeClient, EpisodeRecord, Json, StudyDescriptor, SpatialAxis } from './types';

export class EpisodeView {
  readonly el: HTMLElement;
  readonly replay: ReplayController;
  readonly session: EpisodeSession;
  readonly ready: Promise<void>;
  private catalog: StudyDescriptor[] = [];
  private editor: EpisodeControls | null = null;
  private perspective: Perspective = { kind: 'agent', agent: '0' };
  private busy = false;
  private generation = 0;
  private comparisonIndex: number | null = null;
  private readonly comparisonSlot = h('section', { class: 'episode-comparison' });
  private readonly comparisonControls = h('fieldset', { class: 'row' });
  private comparisonCandidates: Json[] = [];
  private comparisonOptionsKey = '';
  private spatialAxis: SpatialAxis | null = null;
  private notesAxis: NotesAxis = 'arm';
  private axisOptionsKey = '';
  private perspectiveOptionsKey = '';
  private readonly axisPicker = h('select', { 'aria-label': 'Spatial comparison axis', onchange: () => { this.spatialAxis = null; this.notesAxis = this.axisPicker.value === 'routine' ? 'routine' : 'arm'; this.comparisonIndex = null; this.draw(); } });
  private readonly comparisonPicker = h('select', { 'aria-label': 'Matched policy' });
  private readonly comparisonRun = h('button', { onclick: () => {
    const input = this.comparisonCandidates[Number(this.comparisonPicker.value)];
    const record = this.replay.record;
    if (input && record) void this.compare(input, record, this.spatialAxis ?? undefined);
  } }, 'Run matched comparison');
  private readonly comparisonSeek = h('input', { type: 'range', min: 0, 'aria-label': 'Comparison checkpoint', oninput: () => { this.comparisonIndex = Number(this.comparisonSeek.value); this.draw(); } });
  private readonly comparisonCounts = h('p');
  private readonly comparisonName = h('p');
  private readonly comparisonBody = h('div');
  private readonly comparisonTimeline = h('div');
  private disposed = false;
  private pendingInput: Json | undefined;
  private readonly picker = h('select', { 'aria-label': 'Episode study', onchange: () => this.pick() });
  private readonly editorSlot = h('div');
  private readonly setup = h('fieldset', { class: 'episode-setup', disabled: true });
  private readonly status = h('p', { role: 'status', class: 'hint' }, 'Loading study catalog…');
  private readonly exportButton = h('button', { onclick: () => void this.export() }, 'Export shown episode');
  private readonly runButton = h('button', { class: 'primary', onclick: () => void this.run() }, 'Run episode');
  private readonly cancelButton = h('button', { disabled: true, onclick: () => this.client.cancel() }, 'Cancel');
  private readonly stage = h('section', { class: 'episode-rendered' });
  private readonly shownTitle = h('h2', {}, 'Episode');
  private readonly shownNote = h('p', { class: 'hint' }, 'Choose a study and run an episode.');
  private readonly playback = h('fieldset', { class: 'episode-playback', disabled: true });
  private readonly perspectivePicker = h('select', { 'aria-label': 'Perspective', onchange: () => this.changePerspective() });
  private readonly seek = h('input', { type: 'range', min: 0, max: 0, value: 0, 'aria-label': 'Checkpoint', oninput: () => this.replay.seek(Number(this.seek.value)) });
  private readonly counter = h('output', { 'aria-live': 'polite' });
  private readonly playButton = h('button', { onclick: () => { if (this.replay.playing) { this.replay.pause(); this.draw(); } else this.replay.play(); } }, 'Play');
  private readonly retained = h('select', { 'aria-label': 'Retained episode', onchange: () => this.showRecord(this.session.records[Number(this.retained.value)]) });
  private readonly recordedSlot = h('div');
  private readonly recordedButton = h('button', { onclick: () => void this.loadRecorded() }, 'Load recorded study results');
  private readonly fileInput = h('input', { type: 'file', accept: '.json,application/json', hidden: true, onchange: () => void this.openFile() });
  constructor(private client: EpisodeClient = createEpisodeClient()) {
    this.session = new EpisodeSession(client);
    this.replay = new ReplayController(() => this.draw());
    this.setup.append(h('div', { class: 'row' }, h('label', {}, 'Study ', this.picker), h('button', { onclick: () => this.fileInput.click() }, 'Open episode…'), this.fileInput), this.editorSlot, h('div', { class: 'row' }, this.runButton, h('button', { onclick: () => void this.share() }, 'Share input link'), this.exportButton));
    const previous = h('button', { onclick: () => this.replay.step(-1) }, 'Previous');
    const next = h('button', { onclick: () => this.replay.step() }, 'Next');
    this.playback.append(h('div', { class: 'row' }, h('button', { onclick: () => this.replay.reset() }, 'Reset'), previous, this.playButton, next, h('label', {}, 'Perspective ', this.perspectivePicker)), h('div', { class: 'episode-timeline' }, this.seek, this.counter));
    this.comparisonControls.append(h('label', {}, 'Control ', this.axisPicker), h('label', {}, 'Compare with ', this.comparisonPicker), this.comparisonRun);
    this.comparisonTimeline.append(this.comparisonCounts, this.comparisonName, h('div', { class: 'row' }, this.comparisonSeek, h('button', { onclick: () => { this.comparisonIndex = null; this.draw(); } }, 'Join current public clock')), this.comparisonBody);
    this.comparisonSlot.append(h('h3', {}, 'Matched original control comparison'), h('p', { class: 'hint' }, 'Changes one declared original control. Retains geometry, resources, spawn order, goals, seed, horizon and sampling. Same seed does not promise paired random draws. Only compatible choices are listed.'), this.comparisonControls, this.comparisonTimeline);
    this.comparisonSlot.hidden = true;
    this.el = h('div', { class: 'episode-view' }, this.setup, h('div', { class: 'row' }, this.cancelButton, this.status), h('div', { class: 'episode-shown-heading' }, this.shownTitle, this.retained), this.shownNote, this.playback, this.stage, this.comparisonSlot, this.recordedButton, this.recordedSlot);
    this.ready = this.initialize();
  }
  private async initialize(): Promise<void> {
    this.busy = true;
    try {
      this.catalog = (await this.client.catalog()).map(spatialCatalog);
      if (this.disposed) return;
      this.picker.replaceChildren(...this.catalog.map(d => h('option', { value: d.id }, `${d.title}${rendererAvailable(d) ? '' : ' (visualization unavailable)'}`)));
      this.pick(); this.status.textContent = '';
      if (this.pendingInput !== undefined) { const input = this.pendingInput; this.pendingInput = undefined; this.openInput(input); }
    } catch (error) { this.error(error); }
    finally { this.setBusy(false); }
  }
  openInput(input: Json): void {
    checkedInput(input);
    if (!this.catalog.length) { this.pendingInput = structuredClone(input); return; }
    const descriptor = this.catalog.find(d => d.id === obj(input).study);
    if (!descriptor) throw new Error('unknown episode study');
    this.picker.value = descriptor.id;
    this.setEditor(descriptor, input);
    this.status.textContent = 'Input opened for editing. Run episode to evaluate it.';
  }
  private pick(): void { const d = this.catalog.find(d => d.id === this.picker.value); if (d) this.setEditor(d, d.default_input); }
  private setEditor(descriptor: StudyDescriptor, input: Json): void {
    this.editor = new EpisodeControls(descriptor, input, edited => this.session.edit(edited));
    this.session.edit(input);
    this.editorSlot.replaceChildren(h('p', {}, descriptor.question), h('p', { class: 'hint' }, descriptor.supplied), this.editor.el);
    this.runButton.disabled = !rendererAvailable(descriptor);
  }
  private setBusy(on: boolean): void { this.busy = on; this.setup.disabled = on || !this.catalog.length || this.disposed; this.cancelButton.disabled = !on; this.recordedButton.disabled = on || this.disposed; this.comparisonControls.disabled = on || this.disposed; }
  private error(error: unknown): void { if (!this.disposed) this.status.textContent = error instanceof Error ? error.message : String(error); }
  private async run(): Promise<void> {
    if (this.busy || !this.editor) return;
    this.setBusy(true); this.status.textContent = 'Running episode…';
    try { const input = this.editor.input(); this.session.edit(input); const record = await this.session.run(input); if (!this.disposed) { this.showRecord(record); this.status.textContent = 'Episode complete. Playback uses its recorded checkpoints.'; } }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private async openFile(): Promise<void> {
    const file = this.fileInput.files?.[0]; this.fileInput.value = '';
    if (!file || this.busy) return;
    this.setBusy(true); this.status.textContent = 'Validating imported episode…';
    try { const record = await this.session.import(await readEpisodeFile(file)); if (!this.disposed) { this.showRecord(record); this.status.textContent = `Validated ${file.name}`; } }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private async share(): Promise<void> {
    if (!this.editor) return;
    try { const input = this.editor.input(); this.session.edit(input); const token = await encodeEpisode(input); await navigator.clipboard.writeText(`${location.origin}${location.pathname}${location.search}#e=${token}`); this.status.textContent = 'Input link copied.'; } catch (error) { this.error(error); }
  }
  private async export(): Promise<void> {
    if (this.busy || !this.session.shown) return;
    const study = this.session.shown.study;
    if (study === 'persistent_notes' && this.perspective.kind !== 'researcher') { this.status.textContent = 'Complete episode JSON is Researcher-only. Use the projected comic export for this Agent.'; return; }
    this.setBusy(true);
    try { const text = await this.session.export(); if (!this.disposed) downloadText(`${study}-episode.json`, text, 'application/json'); }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private async loadRecorded(): Promise<void> {
    if (this.busy) return;
    this.setBusy(true); this.status.textContent = 'Loading retained measurements…';
    try { const recorded = await this.client.recorded(); if (!this.disposed) { this.recordedSlot.replaceChildren(renderRecorded(recorded)); this.recordedButton.hidden = true; this.status.textContent = 'Retained measurements loaded from their original source.'; } }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private showRecord(record: EpisodeRecord): void {
    this.generation++; this.comparisonIndex = null;
    this.session.shown = record;
    // Do not inspect future local maps merely to populate this checkpoint's perspective menu.
    this.perspective = { kind: 'agent', agent: Object.keys(record.checkpoints[0].local)[0] ?? '0' };
    this.retained.replaceChildren(...this.session.records.map((r, i) => h('option', { value: i, selected: r === record }, `${i + 1}: ${this.catalog.find(d => d.id === r.study)?.title ?? r.study}`)));
    this.replay.load(record);
  }
  private changePerspective(): void { this.perspective = this.perspectivePicker.value === 'researcher' ? { kind: 'researcher' } : { kind: 'agent', agent: this.perspectivePicker.value.slice(6) }; this.draw(); }
  private draw(): void {
    const record = this.replay.record;
    if (!record || this.disposed) return;
    const descriptor = this.catalog.find(d => d.id === record.study);
    if (!descriptor) return;
    const checkpoint = record.checkpoints[this.replay.index];
    const agents = new Set(Object.keys(checkpoint.local));
    for (const row of Array.isArray(obj(checkpoint.public).roster) ? obj(checkpoint.public).roster as Json[] : []) agents.add(String(obj(row).id));
    if (this.perspective.kind === 'agent') agents.add(this.perspective.agent);
    const current = this.perspective.kind === 'researcher' ? 'researcher' : `agent:${this.perspective.agent}`;
    const perspectiveOptionsKey = JSON.stringify([...agents]);
    if (perspectiveOptionsKey !== this.perspectiveOptionsKey) {
      this.perspectiveOptionsKey = perspectiveOptionsKey;
      this.perspectivePicker.replaceChildren(...[...agents].map(agent => h('option', { value: `agent:${agent}`, selected: current === `agent:${agent}` }, `Agent ${agent}`)), h('option', { value: 'researcher', selected: current === 'researcher' }, 'Researcher (privileged)'));
    }
    this.perspectivePicker.value = current;
    this.playback.disabled = false;
    this.seek.max = String(record.checkpoints.length - 1); this.seek.value = String(this.replay.index);
    this.counter.textContent = `${this.replay.index + 1} / ${record.checkpoints.length} — ${label(checkpoint.kind)}`;
    this.playButton.textContent = this.replay.playing ? 'Pause' : 'Play';
    this.playButton.disabled = matchMedia('(prefers-reduced-motion: reduce)').matches;
    this.playButton.title = this.playButton.disabled ? 'Reduced motion: use Previous, Next, or the checkpoint slider.' : 'Play retained checkpoints';
    this.shownTitle.textContent = descriptor.title;
    this.exportButton.textContent = descriptor.family === 'notes' ? 'Export Researcher episode JSON' : 'Export shown episode';
    this.exportButton.disabled = descriptor.family === 'notes' && this.perspective.kind !== 'researcher';
    this.shownNote.textContent = `${descriptor.family === 'notes' ? 'Shown native episode' : 'Shown successful episode'} · ${label(record.semantics)}. Editing controls above changes the next run.`;
    if (record.study === 'key_delegation' && obj(record.input).selector === 'InitiallyEquipped') this.shownNote.textContent += ' Changed initial ownership: Receiver starts with the key. Main-game regrets and equilibrium classification are unavailable. This is a physical control, not a matched causal transfer estimate.';
    const projected = projectCheckpoint(record, this.replay.index, this.perspective);
    const rendered = descriptor.family === 'notes' ? renderNotes(projected, renderDescriptor(descriptor)) : descriptor.family === 'key' ? renderKey(projected, renderDescriptor(descriptor)) : descriptor.family === 'gate' ? renderGate(projected, renderDescriptor(descriptor)) : descriptor.family === 'game' ? renderGame(projected, renderDescriptor(descriptor)) : descriptor.family === 'testimony' ? renderTestimony(projected, renderDescriptor(descriptor)) : descriptor.family === 'spatial' ? (['protection_recaching','deception_gestures'].includes(descriptor.id)?renderCaching:renderSpatial)(projected, renderDescriptor(descriptor)) : renderSurface(projected, renderDescriptor(descriptor));
    const payload = resultPayload(record, this.replay.index, this.perspective);
    const result = h('section', { class: 'episode-result' });
    if (payload !== null) {
      const p = obj(payload);
      result.append(h('div', {}, h('h3', {}, this.perspective.kind === 'researcher' ? 'Researcher — complete episode result' : 'Final episode result'), fact('Availability', descriptor.family === 'surface' ? surfaceSummary(payload).status : p.availability ?? p.status ?? 'complete'), p.expected_payoff !== undefined ? fact('Conditional expected payoff', p.expected_payoff) : null, p.conditional_regret !== undefined ? fact('Conditional expected regret', p.conditional_regret) : null, p.outcome ? fact('Outcome', p.outcome) : null, p.fingerprint ? fact('Fingerprint', p.fingerprint) : null, details('Shown input and rules identity', { input: record.input, rules_identity: record.rules_identity }), p.retained_audit_summary ? details('Retained population and guarantee measurements', p.retained_audit_summary) : null));
      if (descriptor.family === 'notes') result.append(details('Researcher complete native outcome',p.outcome));
      if (descriptor.family === 'key') result.append(renderKeyResult(payload));
      if (descriptor.family === 'gate') result.append(renderGateResult(payload));
      if (descriptor.family === 'surface') result.append(renderSurfaceResult(payload));
      if (descriptor.family === 'spatial') result.append(renderSpatialResult(payload));
      if (record.semantics === 'conditional_case') result.append(h('p', { class: 'hint' }, 'Conditional expectations are not a realized reward. Private truth and signals are unavailable.'));
    } else result.append(h('p', { class: 'hint' }, ['surface_discovery', 'gate_assistance', 'key_delegation', 'persistent_notes'].includes(record.study) ? 'Episode results are available only in Researcher perspective.' : 'Complete results become available at the final checkpoint or in Researcher perspective.'));
    this.stage.replaceChildren(rendered, result);
    if (descriptor.family === 'notes') this.stage.prepend(h('div', {class:'row notes-exports'}, h('button', {onclick:()=>void this.exportComic(false)}, 'Export projected comic SVG'), h('button', {onclick:()=>void this.exportComic(true)}, 'Export projected printable HTML'), h('p', {class:'hint'}, 'Complete episode JSON is available only in Researcher perspective.')));
    this.drawComparison(record, descriptor);
  }
  private async compare(input: Json, original: EpisodeRecord, axis?: SpatialAxis): Promise<void> {
    const key = (value: Json) => original.study === 'persistent_notes' ? notesComparisonKey(value, this.notesAxis) : original.study === 'key_delegation' ? keyComparisonKey(value) : original.study === 'gate_assistance' ? gateComparisonKey(value) : axis ? spatialComparisonKey(value, axis) : comparisonKey(value);
    if (this.busy || key(input) !== key(original.input)) return;
    const generation = this.generation, priorPerspective = this.perspective;
    this.setBusy(true); this.status.textContent = 'Running matched policy…';
    try {
      const next = await this.client.request('run', checkedInput(input));
      if (this.disposed || generation !== this.generation || this.replay.record !== original) return;
      if (key(next.input) !== key(original.input)) throw new Error('Comparison inputs differ');
      this.session.records = [original, next];
      this.showRecord(original);
      if (original.study === 'persistent_notes') { this.perspective = priorPerspective; this.draw(); }
      this.status.textContent = 'Matched episodes ready. Public clocks align only where both records contain the same stage.';
    } catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private drawComparison(record: EpisodeRecord, descriptor: StudyDescriptor): void {
    const notes = descriptor.family === 'notes', spatial = descriptor.family === 'spatial', gate = descriptor.family === 'gate', keyStudy = descriptor.family === 'key';
    this.comparisonSlot.hidden = descriptor.family !== 'surface' && !spatial && !gate && !keyStudy && !notes;
    if (this.comparisonSlot.hidden) return;
    this.axisPicker.parentElement!.hidden = !spatial && !notes;
    this.comparisonSlot.querySelector('h3')!.textContent = notes ? 'Matched Notes arm or routine comparison' : keyStudy ? 'Matched Key schedule comparison' : gate ? 'Matched Gate schedule comparison' : spatial ? 'Matched original control comparison' : descriptor.id === 'surface_discovery' ? 'Matched primary policy comparison' : 'Matched original policy comparison';
    this.comparisonSlot.querySelector('.hint')!.textContent = notes ? 'Changes only arm or routine. Retains stratum, hidden request and identifiers. Native public clocks join only at recorded boundaries. Each pane keeps the selected perspective; two episodes are retained.' : keyStudy ? 'Retains stratum and identifiers. Selector and raw schedules are distinct choices. Initially equipped changes starting ownership and has no main-game classification or matched causal transfer interpretation. At most two episodes are retained.' : gate ? 'Changes only the selector or fixed raw schedule pair. Retains stratum, identifiers and fixed protocol. Team optimum, pure equilibrium and fixed script are distinct labels. Sequential pairs retain at most two episodes.' : spatial ? 'Changes one declared original control. Retains geometry, resources, spawn order, goals, seed, horizon and sampling. Same seed does not promise paired random draws. Only compatible choices are listed.' : descriptor.id === 'surface_discovery' ? 'Changes only the primary policy. Retains mechanism, investigator role, identifiers and complete sequence. Compare the three primary policies as sequential pairs; at most two episodes are retained. KnownRaw and InspectOnly remain labeled controls in the setting selector.' : 'Changes only the declared policy or Agent pair. Retains the mechanism, role, identifiers, priors and complete sequence.';
    if (notes) {
      if (this.axisOptionsKey !== 'notes') { this.axisOptionsKey = 'notes'; this.axisPicker.replaceChildren(h('option',{value:'arm'},'Arm'),h('option',{value:'routine'},'Routine')); this.notesAxis = 'arm'; }
      this.axisPicker.value = this.notesAxis; this.spatialAxis = null;
    } else if (spatial) {
      const axes = obj(descriptor.controls).comparisons;
      const rows = Array.isArray(axes) ? axes.map(obj) : [];
      const axesKey = JSON.stringify(rows);
      if (axesKey !== this.axisOptionsKey) { this.axisOptionsKey = axesKey; this.axisPicker.replaceChildren(...rows.map(a => h('option', { value: a.id }, label(String(a.id))))); this.spatialAxis = null; }
      this.spatialAxis = declaredSpatialAxis(descriptor, this.axisPicker.value);
    } else this.spatialAxis = null;
    const axis = this.spatialAxis;
    const candidates = notes ? notesMatchedInputs(record.input,this.notesAxis) : keyStudy ? keyMatchedInputs(record.input, descriptor) : gate ? gateMatchedInputs(record.input, descriptor) : axis ? spatialMatchedInputs(record.input, descriptor, axis.id) : matchedInputs(record.input, descriptor);
    const name = (input: Json) => notes ? String(obj(input)[this.notesAxis]) : keyStudy ? keySettingLabel(input) : gate ? gateSettingLabel(input) : axis ? `${label(axis.id)}: ${String(spatialAxisValue(input, axis))}` : surfaceSettingLabel(input);
    const key = (input: Json) => notes ? notesComparisonKey(input,this.notesAxis) : keyStudy ? keyComparisonKey(input) : gate ? gateComparisonKey(input) : axis ? spatialComparisonKey(input, axis) : comparisonKey(input);
    const optionsKey = JSON.stringify(candidates);
    // Updating checkpoint content must not detach controls being focused or dragged.
    // Only a changed compatible candidate set replaces options/reset selection.
    if (optionsKey !== this.comparisonOptionsKey) {
      this.comparisonOptionsKey = optionsKey;
      this.comparisonCandidates = candidates;
      this.comparisonPicker.replaceChildren(...candidates.map((input, i) => h('option', { value: i }, name(input))));
    }
    this.comparisonRun.disabled = !candidates.length;
    const other = this.session.records.find(r => r !== record && r.study === record.study && key(r.input) === key(record.input));
    this.comparisonTimeline.hidden = !other;
    this.comparisonBody.replaceChildren();
    if (!other) return;
    const at = record.checkpoints[this.replay.index];
    const index = this.comparisonIndex ?? (notes ? matchNotesCheckpoint(record,other,this.replay.index,this.notesAxis) ?? -1 : keyStudy ? matchKeyCheckpoint(record, other, this.replay.index) ?? -1 : matchingCheckpoint(other.checkpoints, at));
    this.comparisonCounts.textContent = `Independent timelines: shown ${record.checkpoints.length} checkpoints; comparison ${other.checkpoints.length} checkpoints.`;
    this.comparisonName.textContent = name(other.input);
    this.comparisonSeek.max = String(other.checkpoints.length - 1);
    this.comparisonSeek.value = String(Math.max(0, index));
    if (index < 0) {
      this.comparisonBody.append(h('p', { role: 'status' }, 'Unavailable at this public clock and stage. This run ended or did not visit this checkpoint; no wait or state is inferred. Use its independent timeline to inspect it.'));
      return;
    }
    this.comparisonBody.append(h('p', {}, `${index + 1} / ${other.checkpoints.length} · ${this.comparisonIndex === null ? 'Matched public clock and stage' : 'Independent checkpoint'}`), notes ? renderNotes(projectCheckpoint(other,index,this.perspective),renderDescriptor(descriptor)) : keyStudy ? renderKey(projectCheckpoint(other, index, this.perspective), renderDescriptor(descriptor)) : gate ? renderGate(projectCheckpoint(other, index, this.perspective), renderDescriptor(descriptor)) : spatial ? (['protection_recaching','deception_gestures'].includes(descriptor.id)?renderCaching:renderSpatial)(projectCheckpoint(other, index, this.perspective), renderDescriptor(descriptor)) : renderSurface(projectCheckpoint(other, index, this.perspective), renderDescriptor(descriptor)));
    const payload = resultPayload(other, index, this.perspective);
    if (payload !== null) this.comparisonBody.append(h('h3', {}, this.perspective.kind === 'researcher' ? 'Researcher · comparison complete result' : 'Comparison final result'), notes ? details('Researcher complete Notes result',payload) : keyStudy ? renderKeyResult(payload) : gate ? renderGateResult(payload) : spatial ? renderSpatialResult(payload) : renderSurfaceResult(payload));
    else if (['surface_discovery', 'gate_assistance', 'key_delegation', 'persistent_notes'].includes(record.study)) this.comparisonBody.append(h('p', { class: 'hint' }, 'Comparison episode results are available only in Researcher perspective.'));
  }
  private async exportComic(html:boolean):Promise<void> {
    const record=this.replay.record;if(!record||record.study!=='persistent_notes'||this.busy)return;
    // Project before passing any frame into selection, captions, SVG or HTML.
    const perspective={...this.perspective};
    const points=record.checkpoints.map((_,i)=>projectCheckpoint(record,i,perspective));
    const label=perspective.kind==='researcher'?'Researcher (privileged)':`Agent ${perspective.agent}`;
    this.setBusy(true);
    try { const svg=notesComicSvg(points,await embeddedNotesAssets(),label);downloadText(`persistent-notes-${perspective.kind}-comic.${html?'html':'svg'}`,html?notesComicHtml(svg):svg,html?'text/html':'image/svg+xml');this.status.textContent=`Exported ${label} projected comic.`; }
    catch(error){this.error(error);}finally{this.setBusy(false);}
  }
  pause(): void { this.replay.pause(); this.draw(); }
  dispose(): void {
    this.disposed = true;
    this.replay.dispose(); this.session.edit(null); this.session.dispose();
    this.comparisonIndex = null; this.comparisonCandidates = []; this.comparisonOptionsKey = '';
    this.pendingInput = undefined; this.editor = null; this.spatialAxis = null; this.catalog = [];
    this.axisOptionsKey = ''; this.perspectiveOptionsKey = '';
    this.editorSlot.replaceChildren(); this.axisPicker.replaceChildren(); this.perspectivePicker.replaceChildren(); this.retained.replaceChildren(); this.picker.replaceChildren();
    this.stage.replaceChildren(); this.comparisonBody.replaceChildren();
    this.comparisonCounts.textContent = ''; this.comparisonName.textContent = '';
    this.comparisonPicker.replaceChildren(); this.comparisonSlot.hidden = true;
    this.recordedSlot.replaceChildren();
  }
}
