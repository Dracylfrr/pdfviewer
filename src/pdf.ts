import { getDocument, GlobalWorkerOptions } from 'pdfjs-dist';
import worker from 'pdfjs-dist/build/pdf.worker.min.mjs?url';
import { call } from './model';

GlobalWorkerOptions.workerSrc=worker;
export async function loadPdf(id:number) {
  const bytes=await call<ArrayBuffer>('pdf',{id});
  return getDocument({
    data:bytes,cMapUrl:new URL('cmaps/',location.href).href,cMapPacked:true,
    standardFontDataUrl:new URL('standard_fonts/',location.href).href,wasmUrl:new URL('wasm/',location.href).href,
  });
}
