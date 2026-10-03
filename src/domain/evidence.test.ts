import {describe,it,expect} from 'vitest';
import {statusLabels,kindLabels} from './evidence';
describe('visible epistemic contract',()=>{
  it('labels AI separately from observed and derived evidence',()=>{expect(statusLabels.hypothesis).toBe('AI HYPOTHESIS');expect(statusLabels.correlated).toBe('CORRELATION');expect(statusLabels.derived).toContain('DERIVED');expect(statusLabels.observed).toBe('OBSERVED');});
  it('labels blame as attribution rather than causation',()=>{expect(kindLabels.blame_result).toBe('BLAME');expect(Object.values(kindLabels).join(' ')).not.toContain('ROOT CAUSE');});
});
