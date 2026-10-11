import { expect, it } from 'vitest';
import { checkedGateId, gateNumericValue } from './controls';
it('captures valid unblurred raw word IDs exactly for Run and Share', () => {
  expect(checkedGateId('728', 'holder')).toBe(728);
  expect(checkedGateId('63', 'retriever')).toBe(63);
});
it.each(['', '-1', '1.5', '1e2', 'true', '729', ' 1'])('rejects invalid Holder ID %s without replacement', text => {
  expect(() => checkedGateId(text, 'holder')).toThrow('holder');
});
it('does not repair malformed opened native types until explicit edits', () => {
  expect(gateNumericValue('63', '63', false, 'retriever')).toBe('63');
  expect(gateNumericValue(null, '', false, 'holder')).toBeNull();
  expect(gateNumericValue(0, '63', false, 'retriever')).toBe(63);
});
