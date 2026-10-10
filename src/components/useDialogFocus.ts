import {useEffect,useRef} from 'react';
export function useDialogFocus(onClose:()=>void,busy=false){
 const panel=useRef<HTMLElement>(null),close=useRef(onClose),locked=useRef(busy);close.current=onClose;locked.current=busy;
 useEffect(()=>{const previous=document.activeElement as HTMLElement|null;panel.current?.querySelector<HTMLElement>('input,button')?.focus();
  const handle=(e:KeyboardEvent)=>{if(e.key==='Escape'&&!locked.current){e.preventDefault();close.current();}if(e.key!=='Tab')return;
   const controls=Array.from(panel.current?.querySelectorAll<HTMLElement>('button:not(:disabled),input:not(:disabled),select:not(:disabled),[tabindex="0"]')??[]),first=controls[0],last=controls.at(-1);
   if(!controls.length){e.preventDefault();return;}if(e.shiftKey&&document.activeElement===first){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first?.focus();}
  };document.addEventListener('keydown',handle);return()=>{document.removeEventListener('keydown',handle);if(previous?.isConnected)previous.focus();};
 },[]);return panel;
}
