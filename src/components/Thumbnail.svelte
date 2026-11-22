<script lang="ts">
  import type { PDFDocumentProxy } from 'pdfjs-dist';
  import { call, type Entry } from '../model';
  let {pdf, entry, page, go}: {pdf:PDFDocumentProxy;entry:Entry;page:number;go:(target:number)=>void}=$props();
  let preview=$state('');
  $effect(()=>{
    const doc=pdf, number=page, id=entry.id, stamp=entry.modified;
    let active=true, objectUrl='';
    let render:ReturnType<Awaited<ReturnType<typeof doc.getPage>>['render']>|undefined;
    async function getPreview(){
      const bytes=await call<ArrayBuffer>('get_thumbnail',{id,stamp,page:number}).catch(()=>null);
      if(!active)return;
      if(bytes?.byteLength){
        objectUrl=URL.createObjectURL(new Blob([bytes],{type:'image/webp'}));
        preview=objectUrl;
        return;
      }
      const source=await doc.getPage(number);
      if(!active)return;
      const original=source.getViewport({scale:1});
      const viewport=source.getViewport({scale:Math.min(80/original.width,92/original.height)});
      const canvas=document.createElement('canvas');
      canvas.width=Math.ceil(viewport.width);canvas.height=Math.ceil(viewport.height);
      render=source.render({canvas,viewport});
      await render.promise;
      if(!active)return;
      const blob=await new Promise<Blob|null>(resolve=>canvas.toBlob(resolve,'image/webp',0.75));
      if(!active||!blob)return;
      objectUrl=URL.createObjectURL(blob);
      preview=objectUrl;
      const data=[...new Uint8Array(await blob.arrayBuffer())];
      void call('put_thumbnail',{id,stamp,page:number,data}).catch(()=>{});
    }
    getPreview().catch(()=>{});
    return ()=>{active=false;render?.cancel();if(objectUrl)URL.revokeObjectURL(objectUrl)};
  });
</script>
<button class="thumbnail" aria-label={'Go to page '+page} onclick={()=>go(page)}>
  {#if preview}<img src={preview} alt="">{/if}<span>{page}</span>
</button>
