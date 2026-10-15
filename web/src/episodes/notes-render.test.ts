import { expect, it } from 'vitest';
import { notesSceneSvg, notesCaption } from './notes-view';
import { notesPanels, notesComicSvg } from './notes-comic';
import type { Checkpoint,Json } from './types';
const clock={boundary:'0',phase:'Writer',writer_slot:'0',reader_slot:'0'};
const point=(view:Record<string,Json>={},researcher:Json=null):Checkpoint=>({index:0,kind:'initial',clock,public:{writer_active:true,reader_active:true,terminal:false},researcher,local:{Reader:{view:{role:'Reader',cell:'O',key:false,cargo:null,clock,map:{sensed_at:clock,cells:[]},history:[],known_request:null,note_read:null,...view}}}});
it('renders unavailable private request and field without inspecting any unprojected source',()=>{
 const svg=notesSceneSvg(point());expect(svg).toContain('Raw note: Unavailable');expect(svg).toContain('Known target: Unavailable');expect(svg).not.toContain('NoteRight');
});
it('dates observations and removes departed Writer from active maps despite stale occupants',()=>{
 const p=point({map:{sensed_at:clock,cells:[{cell:'O',occupants:['Writer','Reader'],goods:[],key_holder:'Writer'}]}});p.public={writer_active:false,reader_active:true,terminal:false};
 const svg=notesSceneSvg(p);expect(svg).not.toContain('aria-label="Writer Flump"');expect(svg).toContain('Dated map');
});
it('labels Oracle supplied knowledge and separates raw note from target card',()=>{
 const svg=notesSceneSvg(point({reader_routine:'Oracle',known_request:'Right',note_read:{symbol:'Blank',clock}}));expect(svg).toContain('Oracle: supplied target knowledge');expect(svg).toContain('Known target: Right');expect(svg).toContain('Raw note: Blank');
});
it('does not select hidden peer action boundaries for an Agent comic',()=>{
 const a=point(),b=point();b.index=1;b.clock={...clock,boundary:'1'};const panels=notesPanels([a,b]);expect(panels).toHaveLength(6);expect(panels[0].point).toBeNull();expect(notesComicSvg([a,b],{},'Agent Reader')).not.toContain('Right');
});
it('selects actual own acquisition/action clocks and preserves exact cost captions',()=>{
 const p=point({history:[{clock,primitive:{kind:'InspectManifest'},cost:'4',status:'Applied',observed_request:'Right'}]});expect(notesPanels([p])[0].point).toBe(p);expect(notesCaption(p)).toContain('cost 4');
});
it('escapes opaque identity text in SVG and never embeds full checkpoint metadata',()=>{
 const p=point();p.local={'<Reader>':p.local.Reader};const out=notesComicSvg([p],{},'Agent <Reader>');expect(out).toContain('Agent &lt;Reader&gt;');expect(out).not.toContain('<metadata');
});
it('keeps unused staging distinct from the C goods and manifest inspection site',()=>{
 const svg=notesSceneSvg(point());expect(svg).toContain('Unused staging');expect(svg).toContain('Goods / manifest');expect(svg).not.toContain('Private target');
});
it('dates each retained cell independently of the last local-map update',()=>{
 const p=point({map:{sensed_at:{...clock,boundary:'4'},cells:[{cell:'C',sensed_at:{...clock,boundary:'1'},occupants:['Writer'],goods:[]}]}});
 expect(notesSceneSvg(p)).toContain('Dated b1 occupants: Writer');
});
