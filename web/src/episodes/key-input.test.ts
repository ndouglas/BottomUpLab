import { expect, it } from 'vitest';
import { checkedKeyId } from './key-controls';
it.each(['', 'true','false','0x10','1.5','1e2',' 1','01','-1','729'])('rejects noncanonical or out-of-range giver text %s', text=>expect(()=>checkedKeyId(text,'giver')).toThrow('giver'));
it('checks role-specific bounds and valid canonical decimal',()=>{expect(checkedKeyId('728','giver')).toBe(728);expect(checkedKeyId('63','receiver')).toBe(63);expect(checkedKeyId('0','receiver')).toBe(0);expect(()=>checkedKeyId('64','receiver')).toThrow('receiver');});
