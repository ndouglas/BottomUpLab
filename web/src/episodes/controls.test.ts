import { expect, it } from 'vitest';
import { checkedSurfaceSequence } from './controls';
it('captures an unblurred complete sequence exactly', () => {
  expect(checkedSurfaceSequence('255')).toBe(255);
});
it.each(['', 'x', '1.5', '-1', '256', '1e2'])('rejects invalid sequence %s without silently using zero', value => {
  expect(() => checkedSurfaceSequence(value)).toThrow('sequence: enter a whole number from 0 to 255');
});
import { surfaceSequenceValue } from './controls';
it('preserves an opened string sequence for exact native validation until explicitly edited', () => {
  expect(surfaceSequenceValue('255', '255', false)).toBe('255');
  expect(surfaceSequenceValue('255', '255', true)).toBe(255);
});
it('preserves an opened null sequence and captures edits on an original numeric sequence', () => {
  expect(surfaceSequenceValue(null, '', false)).toBeNull();
  expect(surfaceSequenceValue(0, '255', false)).toBe(255);
});
