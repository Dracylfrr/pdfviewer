<script lang="ts">
  import { call, type Entry } from '../model';
  let {entry}: {entry:Entry} = $props();
  let box=$state<HTMLDivElement>();
  let image=$state('');
  // One uncached PDF at a time, even when several cards enter the viewport together.
  const tasks = queue();
  $effect(()=>{
    const element=box, {id,modified}=entry;
    if(!element)return;
    let active=true, url='';
    let task:Awaited<ReturnType<typeof import('../pdf').loadPdf>>|undefined;
    const observer=new IntersectionObserver(entries=>{
      if(!entries.some(item=>item.isIntersecting))return;
      observer.disconnect();
      tasks.add(async()=>{
        try {
          if(!active)return;
          const args={id,stamp:modified,page:1};
          const cached=await call<ArrayBuffer>('get_thumbnail',args).catch(()=>null);
          if(!active)return;
          let blob:Blob|null;
          if(cached?.byteLength)blob=new Blob([cached],{type:'image/webp'});
          else {
            const {loadPdf}=await import('../pdf');
            if(!active)return;
            task=await loadPdf(id);
            if(!active)return;
            task.onPassword=()=>{void task?.destroy()};
            const pdf=await task.promise;
            const page=await pdf.getPage(1);
            if(!active)return;
            const original=page.getViewport({scale:1});
            const viewport=page.getViewport({scale:Math.min(80/original.width,92/original.height)});
            const canvas=document.createElement('canvas');
            canvas.width=Math.ceil(viewport.width);canvas.height=Math.ceil(viewport.height);
            await page.render({canvas,viewport}).promise;
            blob=await new Promise<Blob|null>(resolve=>canvas.toBlob(resolve,'image/webp',0.75));
            if(blob&&active){
              const data=[...new Uint8Array(await blob.arrayBuffer())];
              void call('put_thumbnail',{...args,data}).catch(()=>{});
            }
          }
          if(blob&&active){url=URL.createObjectURL(blob);image=url}
        } catch {
          // Keep the book icon for password-protected or unavailable previews.
        } finally { await task?.destroy().catch(()=>{}); }
      });
    });
    observer.observe(element);
    return ()=>{observer.disconnect();active=false;void task?.destroy().catch(()=>{});if(url)URL.revokeObjectURL(url)};
  });
</script>
<script lang="ts" module>
  let pending=Promise.resolve();
  function queue(){
    return {add(job:()=>Promise<void>){pending=pending.then(job,job)}};
  }
</script>
<div class="recent-preview" bind:this={box}>
  {#if image}<img src={image} alt="">{:else}<img class="book-icon" src="/book.svg" alt="">{/if}
</div>
