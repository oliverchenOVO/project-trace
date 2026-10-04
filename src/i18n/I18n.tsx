import {createContext,useContext,useState,useMemo,useEffect,useCallback,type ReactNode} from 'react';
import {initialLanguage,LANGUAGE_KEY,translate,translateEvidence,type Language} from './translate';
type I18nValue = {language:Language;locale:string;setLanguage:(language:Language)=>void;t:(text:string,...values:unknown[])=>string;evidenceText:(e:{kind:string},text:string)=>string};
const I18nContext=createContext<I18nValue|null>(null);
export function I18nProvider({children}:{children:ReactNode}) {
  const [language,setLanguage]=useState<Language>(()=>{
    // Accessing localStorage itself can throw, before getItem is called.
    try { return initialLanguage(window.localStorage,navigator.language); } catch { return navigator.language.toLowerCase().startsWith('zh')?'zh-TW':'en'; }
  });
  useEffect(()=>{document.documentElement.lang=language==='en'?'en':'zh-Hant';try{window.localStorage.setItem(LANGUAGE_KEY,language);}catch{/* Switching still works for this session. */}},[language]);
  const t=useCallback((text:string,...values:unknown[])=>translate(language,text,...values),[language]);
  const evidenceText=useCallback((e:{kind:string},text:string)=>translateEvidence(language,e,text),[language]);
  const value=useMemo(()=>({language,locale:language==='en'?'en-US':'zh-TW',setLanguage,t,evidenceText}),[language,t,evidenceText]);
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}
export function useI18n() { const context=useContext(I18nContext);if(!context)throw Error('I18nProvider is required');return context; }
