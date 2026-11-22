<script lang="ts">
  import { TextLayer, type PDFDocumentProxy } from 'pdfjs-dist';
  import type { Note, Rect } from '../model';
  let {pdf,page,width,height,theme,preserve,notes,error,activate}: {
    pdf:PDFDocumentProxy;page:number;width:number;height:number;theme:string;preserve:boolean;
    notes:Note[];error:(message:string)=>void;activate:(id:string)=>void
  } = $props();
  let canvas:HTMLCanvasElement;
  let textLayer:HTMLDivElement;
  function rectangles(value:string):Rect[] {
    try { return JSON.parse(value) as Rect[]; } catch { return []; }
  }
  $effect(()=>{
    const doc=pdf, number=page, w=width, h=height;
    let cancelled=false;
    let render:ReturnType<Awaited<ReturnType<typeof doc.getPage>>['render']>|undefined;
    let layer:TextLayer|undefined;
    async function draw(){
      const p=await doc.getPage(number);
      if(cancelled)return;
      const original=p.getViewport({scale:1});
      const viewport=p.getViewport({scale:Math.min(w/original.width,h/original.height)});
      const ratio=Math.min(devicePixelRatio,2,Math.sqrt(12_000_000/(viewport.width*viewport.height)));
      canvas.width=Math.round(viewport.width*ratio);
      canvas.height=Math.round(viewport.height*ratio);
      canvas.style.width=viewport.width+'px'; canvas.style.height=viewport.height+'px';
      canvas.parentElement!.style.width=viewport.width+'px'; canvas.parentElement!.style.height=viewport.height+'px';
      textLayer.replaceChildren();
      textLayer.style.setProperty('--total-scale-factor',String(viewport.scale));
      render=p.render({canvas,viewport,transform:[ratio,0,0,ratio,0,0]});
      layer=new TextLayer({textContentSource:p.streamTextContent(),container:textLayer,viewport});
      await Promise.all([render.promise,layer.render()]);
      p.cleanup();
    }
    draw().catch(e=>{if(!cancelled)error('Unable to render page '+number+'. '+String(e))});
    return ()=>{cancelled=true;render?.cancel();layer?.cancel()};
  });
</script>
<div class="page-sheet" data-page={page} style:--paper={theme} class:preserve class:dark-paper={theme==="#322f2c"||theme==="#101010"}>
  <canvas bind:this={canvas} aria-label={'PDF page '+page}></canvas>
  <div class="textLayer" bind:this={textLayer}></div>
  {#each notes as note (note.id)}
    {#each rectangles(note.rects) as rect}
      <button class="annotation-mark" class:underline={note.kind==='underline'} aria-label={'Show '+note.kind} title={note.quote}
        style:left={rect.x*100+'%'} style:top={rect.y*100+'%'} style:width={rect.w*100+'%'} style:height={rect.h*100+'%'}
        style:--mark={note.color} onclick={()=>activate(note.id)}></button>
    {/each}
  {/each}
</div>
