import { invoke } from '@tauri-apps/api/core';
export const call = invoke;
export type Library = { id: number; root: string };
export type Entry = {id:number; library:number; path:string; parent:string; name:string; folder:boolean; size:number; modified:number; missing:boolean; fingerprint:string|null};
export type Hit = {entry:Entry; page:number|null; snippet:string};
export type Rect = {x:number;y:number;w:number;h:number};
export type Note = {id:string;entry:number;kind:string;page:number|null;quote:string;content:string;color:string;rects:string};
export type Reading = {page:number;zoom:number;mode:string;theme:string};
export const defaultReading: Reading = {page:1,zoom:0,mode:'continuous',theme:'Original'};
const sorter = new Intl.Collator(undefined,{numeric:true,sensitivity:'base'});
export const children = async (library:number,parent:string) =>
  (await call<Entry[]>('children',{library,parent})).sort((a,b)=>Number(b.folder)-Number(a.folder)||sorter.compare(a.name,b.name));
export const baseName = (path:string) => path.replace(/[\\/]$/,'').split(/[\\/]/).pop() || path;
export const readerThemes: Record<string,string> = {
  Original:'#ffffff','Paper White':'#fbfaf5','Warm Cream':'#f6efd9',Sepia:'#e8d5ad',Charcoal:'#322f2c','True Black':'#101010',
  'Cloud Blue':'#ddebf7',Lavender:'#e9e1f5',Sage:'#e1ebdd',Mint:'#d8f1e7','Blush Pink':'#f6e2e8',Peach:'#f8e6d8','Soft Gray':'#e7e7e7',
};
export const colors = {Yellow:'#f5d94c',Mint:'#69e4b0',Blue:'#73b2fa',Lavender:'#b899f4',Pink:'#f495bf',Peach:'#ffb17f'};
export const kinds: Record<string,string> = {highlight:'Highlight',underline:'Underline',bookmark:'Bookmark',document:'Document note',selection:'Selection note',folder:'Folder note'};
