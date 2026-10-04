import {describe,it,expect} from 'vitest';
import {parseArgs,runLabels,boundaryWording} from './types';
describe('verification presentation contract',()=>{
 it('keeps structured arguments separate without shell splitting',()=>{expect(parseArgs('["--test", "file with spaces.mjs", "$(data)"]')).toEqual(['--test','file with spaces.mjs','$(data)']);expect(()=>parseArgs('npm test')).toThrow();expect(()=>parseArgs('[1]')).toThrow();});
 it('keeps cancellation, inconclusive, interruption and confirmation distinct',()=>{expect(runLabels.cancelled).toBe('CANCELLED');expect(runLabels.inconclusive).toBe('INCONCLUSIVE');expect(runLabels.interrupted).toContain('INTERRUPTED');expect(runLabels.bisect_confirmed).toBe('BISECT CONFIRMED');});
 it('scopes bisect language to selected predicate and range',()=>{expect(boundaryWording('abc')).toContain('selected test');expect(boundaryWording('abc')).toContain('good/bad range');expect(boundaryWording('abc')).not.toMatch(/causes|root cause proven/i);});
});
