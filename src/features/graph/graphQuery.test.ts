import {it,expect} from 'vitest';
import type {Evidence,Investigation} from '../../domain/evidence';
import {matchesEvidence,selectedPath} from './graphQuery';
it('matches multiple terms across titles, IDs and paths while respecting kind',()=>{
 const e={id:'abc-123',kind:'source_line',title:'Parser null user',summary:'failure',provenance:{filePath:'src/parser.ts'}} as Evidence;
 expect(matchesEvidence(e,'USER src/parser','source_line')).toBe(true);expect(matchesEvidence(e,'abc','commit')).toBe(false);expect(matchesEvidence(e,'missing','')).toBe(false);
 expect(matchesEvidence(e,'原始碼 parser','source_line','原始碼位置')).toBe(true);
});
it('finds ancestors and immediate descendants without spreading into sibling branches or looping on cycles',()=>{
 const c={edges:[{source:'a',target:'b'},{source:'b',target:'c'},{source:'c',target:'b'},{source:'a',target:'sibling'},{source:'b',target:'d'},{source:'d',target:'grandchild'}]} as Investigation;
 expect([...selectedPath(c,'b')].sort()).toEqual(['a','b','c','d']);expect(selectedPath(c,null).size).toBe(0);
});
