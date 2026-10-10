import {describe,it,expect} from 'vitest';
import {filterCases} from './caseSearch';
const cases=[{id:'abc-123',title:'Null User failure',createdAt:1},{id:'def-456',title:'空值處理錯誤',createdAt:2}];
describe('case search',()=>{
 it('finds managed tags and original titles after a rename',()=>{const saved=[{...cases[0],title:'作品展示',originalTitle:'Null User failure',tags:['Rust','推甄']}];expect(filterCases(saved,'user rust')).toEqual(saved);expect(filterCases(saved,'推甄')).toEqual(saved);});
 it('matches multiple terms case-insensitively across title and ID',()=>{expect(filterCases(cases,'  USER abc  ')).toEqual([cases[0]]);expect(filterCases(cases,'空值')).toEqual([cases[1]]);expect(filterCases(cases,'user def')).toEqual([]);});
 it('preserves the full list and source order when cleared',()=>{expect(filterCases(cases,'  ')).toBe(cases);expect(filterCases(cases,'missing')).toEqual([]);expect(cases).toHaveLength(2);});
});
