import {describe,it,expect} from 'vitest';
import {readFileSync,globSync} from 'node:fs';
import ts from 'typescript';
import zh from './zh-TW.json';
import {initialLanguage,translate,translateEvidence,LANGUAGE_KEY} from './translate';
describe('interface language',()=>{
  it('retains explicit choice independently of browser language',()=>{
    const storage={getItem:(key:string)=>key===LANGUAGE_KEY?'en':null};
    expect(initialLanguage(storage,'zh-TW')).toBe('en');
    expect(initialLanguage({getItem:()=> 'zh-TW'},'en-US')).toBe('zh-TW');
  });
  it('uses supported browser defaults when storage is absent, invalid or blocked',()=>{
    expect(initialLanguage({getItem:()=>null},'zh-HK')).toBe('zh-TW');
    expect(initialLanguage({getItem:()=> 'other'},'fr-FR')).toBe('en');
    expect(initialLanguage({getItem:()=>{throw Error('blocked');}},'zh-TW')).toBe('zh-TW');
  });
  it('retains the full SHA and test scope in translated dynamic boundary wording',()=>{
    const sha='e2a7cc531eac0fdd07eebfce3fa9132c63f8da00';
    const english=`The selected test first fails at commit ${sha} within the selected good/bad range. This verifies a regression boundary for the selected test predicate.`;
    const chinese=translate('zh-TW',english);
    expect(chinese).toContain(sha);expect(chinese).toContain('所選測試判定條件');expect(chinese).not.toContain('根本原因已證明');
    expect(translate('en',english)).toBe(english);
  });
  it('inserts arbitrary evidence values once without interpreting nested placeholders',()=>{
    expect(translate('zh-TW','Case: {0}','raw {1}')).toBe('案件：raw {1}');
  });
  it('preserves raw commit, AI, incident and source text even if it matches a UI label',()=>{
    for(const kind of ['commit','hypothesis','incident','source_line','stack_frame'])expect(translateEvidence('zh-TW',{kind},'COMMAND')).toBe('COMMAND');
    expect(translateEvidence('zh-TW',{kind:'regression_boundary'},'REGRESSION BOUNDARY VERIFIED')).toBe('回歸邊界已驗證');
  });
  it('covers all static UI translation keys without translating technical identifiers',()=>{
    const technical=new Set(['TRACE','SHA','SHA-256','HEAD','UTF-8','KB','ms','C:\\\\projects\\\\my-repository']);
    const missing=new Set<string>();
    for(const file of globSync('src/**/*.tsx')){
      const source=ts.createSourceFile(file,readFileSync(file,'utf8'),ts.ScriptTarget.Latest,true,ts.ScriptKind.TSX);
      function visit(n:ts.Node){if(ts.isCallExpression(n)&&n.expression.getText(source)==='t'&&n.arguments[0]&&ts.isStringLiteral(n.arguments[0])){
        const key=n.arguments[0].text.trim().replace(/\s+/g,' ');
        if(/[a-zA-Z]/.test(key)&&!technical.has(key)&&!Object.hasOwn(zh,key))missing.add(key);
      }ts.forEachChild(n,visit);}visit(source);
    }
    expect([...missing]).toEqual([]);
  });
});
