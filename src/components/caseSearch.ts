import type {SavedCase} from '../domain/evidence';
export function filterCases(cases:SavedCase[],query:string):SavedCase[]{
 const terms=query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
 return terms.length?cases.filter(c=>terms.every(term=>`${c.title} ${c.id}`.toLocaleLowerCase().includes(term))):cases;
}
