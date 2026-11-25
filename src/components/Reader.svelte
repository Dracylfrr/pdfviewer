<script lang="ts">
  import { tick, untrack } from 'svelte';
  import type { PDFDocumentProxy } from 'pdfjs-dist';
  import { loadPdf } from '../pdf';
  import { printPdf } from '../print-pdf';

  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { call, colors, defaultReading, readerThemes, type Entry, type Note, type Reading, type Rect } from '../model';
  import Page from './Page.svelte';
  import Thumbnail from './Thumbnail.svelte';
  import Rows from './Rows.svelte';

  let {entry,notes,requested,add,activate,error,info}:{
    entry:Entry;notes:Note[];requested:{page:number;token:number}|null;
    add:(kind:string,page:number|null,quote?:string,color?:string,rects?:Rect[])=>void;
    activate:(id:string)=>void;error:(message:string)=>void;info:()=>void
  }=$props();
  let pdf=$state.raw<PDFDocumentProxy|null>(null);
  let readingState=$state<Reading>({...defaultReading});
  let failure=$state('');
  let outline=$state<{title:string;page:number}[]>([]);
  let thumbnails=$state(false);
  let allPages=$derived(Array.from({length:pdf?.numPages??0},(_,i)=>i+1));
  let showOutline=$state(false);
  let preserve=$state(localStorage.getItem('preserve-colors')==='true');
  let surface=$state<HTMLDivElement>();
  let searchBox=$state<HTMLInputElement>();
  let width=$state(0), height=$state(0), top=$state(0);
  let initialized=$state(false);
  let jump=$state(1);
  let query=$state(''), searching=$state(false);
  let searchToken=0;
  let matches=$state<{page:number;snippet:string}[]>([]);
  let selection=$state<{page:number;quote:string;rects:Rect[];left:number;top:number}|null>(null);
  let color=$state(colors.Yellow);
  let history:number[]=[];
  let historyIndex=-1;
  let previousStride=0;
  let pageRatio=$state(1.414);
  let printDialog=$state<HTMLDialogElement>();
  let printFrom=$state(1), printTo=$state(1), printing=$state(false), printError=$state("");
  let preparedPages=$state(0);
  let printJob:AbortController|undefined;
  let zoomDraft=$state<number|null>(null);
  let zoomTimer:ReturnType<typeof setTimeout>|undefined;
  let notePages=$derived(Map.groupBy(notes.filter(n=>n.page),n=>n.page));
  const baseWidth=750;
  let availableWidth=$derived(Math.max(100,width-40));
  let availableHeight=$derived(Math.max(100,height-42));
  let fittedWidth=$derived(readingState.zoom===-1?Math.min(availableWidth,availableHeight/pageRatio):availableWidth);
  let zoomPercent=$derived(zoomDraft??Math.round(100*(readingState.zoom>0?readingState.zoom:fittedWidth/baseWidth)));
  let pageWidth=$derived(Math.round(readingState.zoom>0?baseWidth*readingState.zoom:fittedWidth));
  let pageHeight=$derived(Math.round(pageWidth*pageRatio));
  let stride=$derived(pageHeight+24);
  let start=$derived(readingState.mode==='single'?readingState.page:Math.max(1,Math.floor(top/stride)-1));
  let end=$derived(readingState.mode==='single'?readingState.page:Math.min(pdf?.numPages??0,Math.floor((top+height)/stride)+2));
  let visible=$derived(Array.from({length:Math.max(0,end-start+1)},(_,i)=>start+i));
  $effect(()=>{
    const id=entry.id;
    let cancelled=false;
    let task:Awaited<ReturnType<typeof loadPdf>>|undefined;
    async function load(){
      try {
        const [loading,saved]=await Promise.all([loadPdf(id),call<Reading|null>('reading',{id})]);
        task=loading;
        if(cancelled){void task.destroy();return}
        task.onPassword=(submit:(password:string)=>void,reason:number)=>{
          const password=window.prompt(reason===2?'Incorrect password. Try again.':'Enter the PDF password.');
          if(password===null){failure='Opening this PDF requires a password.';void task?.destroy();return;}
          submit(password);
        };
        const document=await task.promise;
        const firstPage=await document.getPage(1);
        if(cancelled)return;
        const dimensions=firstPage.getViewport({scale:1});
        pageRatio=dimensions.height/dimensions.width;
        readingState={...defaultReading,...saved,page:Math.min(document.numPages,Math.max(1,saved?.page??1))};
        pdf=document; jump=readingState.page;
        await tick();
        if(cancelled)return;
        navigate(readingState.page,true);
        initialized=true;
        void call("mark_opened",{id}).catch(e=>error(String(e)));
      } catch (e){if(!cancelled&&!failure)failure='Unable to render this PDF. '+String(e)}
    }
    load();
    return ()=>{cancelled=true;++searchToken;clearTimeout(zoomTimer);printJob?.abort();void task?.destroy()};
  });
  $effect(()=>{
    if(!initialized)return;
    const id=entry.id, snapshot=$state.snapshot(readingState);
    const timer=setTimeout(()=>call('save_reading',{id,state:snapshot}).catch(e=>error(String(e))),180);
    return ()=>{
      clearTimeout(timer);
      void call('save_reading',{id,state:snapshot}).catch(e=>error(String(e)));
    };
  });
  $effect(()=>{if(initialized&&requested){const page=requested.page;untrack(()=>navigate(page,true))}});
  $effect(()=>{
    const size=stride;
    if(!surface||!pdf||readingState.mode==='single')return;
    const offset=untrack(()=>top);
    if(previousStride&&previousStride!==size){
      surface.scrollTop=offset/previousStride*size;
      top=surface.scrollTop;
    }
    previousStride=size;
  });
  export async function flushProgress(){
    if(initialized)await call("save_reading",{id:entry.id,state:$state.snapshot(readingState)});
  }
  function navigate(number:number,record=false){
    if(!pdf)return;
    const target=Math.min(pdf.numPages,Math.max(1,Math.trunc(number)||1));
    if(record&&history[historyIndex]!==target){
      history=history.slice(0,historyIndex+1);history.push(target);historyIndex=history.length-1;
    }
    readingState.page=jump=target;
    if(surface){surface.scrollTop=readingState.mode==='single'?0:(target-1)*stride;top=surface.scrollTop;}
  }
  function moveHistory(delta:number){
    const next=historyIndex+delta;
    if(next>=0&&next<history.length){historyIndex=next;navigate(history[next]);}
  }
  function scroll(e:Event){
    top=(e.currentTarget as HTMLDivElement).scrollTop;
    if(!pdf||readingState.mode==='single')return;
    const current=Math.min(pdf.numPages,Math.floor((top+Math.min(80,height/4))/stride)+1);
    if(readingState.page!==current)readingState.page=jump=current;
  }
  async function find(){
    if(!pdf||!query.trim())return;
    const token=++searchToken, doc=pdf, needle=query.trim().toLocaleLowerCase();
    matches=[];searching=true;
    try{
      for(let page=1;page<=doc.numPages&&token===searchToken;page++){
        const data=await (await doc.getPage(page)).getTextContent();
        if(token!==searchToken)return;
        const contents=data.items.map(item=>'str' in item?item.str:'').join(' ');
        const at=contents.toLocaleLowerCase().indexOf(needle);
        if(at>=0)matches=[...matches,{page,snippet:contents.slice(Math.max(0,at-24),at+needle.length+35)}];
        if(matches.length>=100)break;
      }
    }catch(e){if(token===searchToken)error(String(e))}
    finally{if(token===searchToken)searching=false}
  }
  async function loadOutline(){
    showOutline=!showOutline;
    const doc=pdf;
    if(!showOutline||!doc||outline.length)return;
    try {
      const nodes=await doc.getOutline();
      type Node=NonNullable<typeof nodes>[number];
      async function walk(items:Node[],depth:number){
        for(const item of items){
          const dest=typeof item.dest==='string'?await doc!.getDestination(item.dest):item.dest;
          const ref=dest?.[0];
          if(ref!=null){
            const page=typeof ref==='number'?ref+1:await doc!.getPageIndex(ref)+1;
            outline=[...outline,{title:'　'.repeat(depth)+item.title,page}];
          }
          await walk(item.items,depth+1);
        }
      }
      await walk(nodes??[],0);
    }catch(e){error(String(e))}
  }
  function saveSelection(kind:string){
    if(!selection)return;
    add(kind,selection.page,selection.quote,color,selection.rects);
    window.getSelection()?.removeAllRanges();selection=null;
  }
  function selectText(){
    const selected=window.getSelection();
    if(!selected||selected.isCollapsed||!selected.rangeCount){selection=null;return}
    const range=selected.getRangeAt(0);
    const anchor=range.startContainer.parentElement?.closest<HTMLElement>('.page-sheet');
    if(!anchor||range.endContainer.parentElement?.closest('.page-sheet')!==anchor){selection=null;return}
    const bounds=anchor.getBoundingClientRect();
    const rects=Array.from(range.getClientRects()).filter(r=>r.width>1&&r.height>1)
      .map(r=>({x:(r.left-bounds.left)/bounds.width,y:(r.top-bounds.top)/bounds.height,w:r.width/bounds.width,h:r.height/bounds.height}));
    const quote=selected.toString().trim();
    const position=range.getBoundingClientRect();
    selection=rects.length&&quote?{page:Number(anchor.dataset.page),quote,rects,left:Math.max(8,Math.min(position.left,window.innerWidth-380)),top:Math.max(10,position.top-46)}:null;
  }
  function setZoom(percent:number){
    clearTimeout(zoomTimer);
    zoomDraft=null;
    readingState.zoom=Math.max(.25,Math.min(3,percent/100));
  }
  function previewZoom(percent:number){
    zoomDraft=percent;
    clearTimeout(zoomTimer);
    zoomTimer=setTimeout(()=>setZoom(percent),85);
  }
  function fit(mode:number){
    clearTimeout(zoomTimer);
    zoomDraft=null;
    readingState.zoom=mode;
  }
  function showPrint(){
    if(!pdf)return;
    printFrom=1;
    printTo=pdf.numPages;
    printError='';
    printDialog?.showModal();
  }
  async function submitPrint(event:SubmitEvent){
    event.preventDefault();
    if(!pdf||printing)return;
    if(!Number.isInteger(printFrom)||!Number.isInteger(printTo)||printFrom<1||printTo<printFrom||printTo>pdf.numPages){
      printError='Enter a valid page range.';
      return;
    }
    const job=new AbortController();
    printJob=job;printError='';printing=true;preparedPages=0;
    try {
      await printPdf(pdf,printFrom,printTo,job.signal,count=>preparedPages=count,async()=>{
        printDialog?.close();
        await new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve())));
      });
    } catch {
      if(job.signal.aborted)printDialog?.close();
      else {
        printError='Couldn’t open the print dialog for these pages.';
        if(!printDialog?.open)printDialog?.showModal();
      }
    } finally {printing=false;printJob=undefined}
  }
  function keyboard(e:KeyboardEvent){
    if(!pdf||e.defaultPrevented)return;
    if(e.ctrlKey&&e.key.toLowerCase()==='f'){e.preventDefault();searchBox?.focus();return}
    if(e.ctrlKey&&e.key.toLowerCase()==='p'){e.preventDefault();if(!printDialog?.open)showPrint();return}
    if(e.key==='Escape'){selection=null;return}
    if((e.target as HTMLElement)?.matches('input,textarea,select,button,summary,[role=separator],[contenteditable]'))return;
    if(e.ctrlKey&&e.key.toLowerCase()==='b'){e.preventDefault();add('bookmark',readingState.page)}
    if(e.ctrlKey&&['+','=','-','0'].includes(e.key)){e.preventDefault();if(e.key==='0')fit(0);else setZoom(zoomPercent+(e.key==='-'?-10:10));}
    if(!e.ctrlKey&&e.key.toLowerCase()==='h'&&selection){e.preventDefault();saveSelection('highlight')}
    if(e.key==='ArrowLeft'||e.key==='PageUp'){e.preventDefault();navigate(readingState.page-1,true)}
    if(e.key==='ArrowRight'||e.key==='PageDown'){e.preventDefault();navigate(readingState.page+1,true)}
  }
</script>
<svelte:window onkeydown={keyboard} onpointerup={selectText}/>
<div class="reader">
  {#if failure}<div class="empty"><h2>Couldn’t open this PDF</h2><p>{failure}</p></div>
  {:else if !pdf}<div class="empty" role="status">Opening {entry.name}…</div>
  {:else}
    <div class="reader-title"><strong title={entry.name}>{entry.name}</strong><div><button onclick={showPrint}>Print</button> <button onclick={info}>Document info</button></div></div>
    <dialog class="print-dialog" bind:this={printDialog} aria-labelledby="print-title" oncancel={event=>{if(printing)event.preventDefault()}}>
      <form onsubmit={submitPrint}>
        <h2 id="print-title">Print PDF</h2>
        <p class="muted">All pages are selected by default. Change the range to print fewer pages.</p>
        <div class="print-range">
          <label>From page <input type="number" min="1" max={pdf.numPages} required bind:value={printFrom} disabled={printing}></label>
          <label>To page <input type="number" min="1" max={pdf.numPages} required bind:value={printTo} disabled={printing}></label>
        </div>
        {#if printError}<p role="alert">{printError}</p>{/if}
        {#if printing}<p role="status" class="muted">Preparing pages: {preparedPages} of {printTo-printFrom+1}</p>{/if}
        <div class="print-actions">
          <button type="button" onclick={()=>printing?printJob?.abort():printDialog?.close()}>Cancel</button>
          <button class="primary" disabled={printing}>{printing?'Preparing pages…':'Continue to print'}</button>
        </div>
      </form>
    </dialog>
    <div class="reader-tools">
      <button aria-label="Back in reading history" onclick={()=>moveHistory(-1)}>⟲</button>
      <button aria-label="Forward in reading history" onclick={()=>moveHistory(1)}>⟳</button>
      <button aria-label="Previous page" onclick={()=>navigate(readingState.page-1,true)}>‹</button>
      <form onsubmit={e=>{e.preventDefault();navigate(jump,true)}}><input aria-label="Page" class="page-number" type="number" min="1" max={pdf.numPages} bind:value={jump}> <span>/ {pdf.numPages}</span></form>
      <button aria-label="Next page" onclick={()=>navigate(readingState.page+1,true)}>›</button>
      <select aria-label="Page layout" bind:value={readingState.mode} onchange={()=>tick().then(()=>navigate(readingState.page))}><option value="continuous">Continuous</option><option value="single">Single page</option></select>
      <div class="zoom-tools" role="group" aria-label="Page scale">
        <button aria-label="Zoom out" onclick={()=>setZoom(zoomPercent-10)}>−</button>
        <label class="zoom-slider">
          <input type="range" aria-label="Viewing scale" aria-valuetext={zoomPercent+' percent'} min="25" max="300" step="1"
            value={Math.max(25,Math.min(300,zoomPercent))} oninput={event=>previewZoom(event.currentTarget.valueAsNumber)}
            onchange={event=>setZoom(event.currentTarget.valueAsNumber)}>
          <output>{zoomPercent}%</output>
        </label>
        <button aria-label="Zoom in" onclick={()=>setZoom(zoomPercent+10)}>+</button>
        <button aria-pressed={readingState.zoom===0} onclick={()=>fit(0)}>Fit width</button>
        <button aria-pressed={readingState.zoom===-1} onclick={()=>fit(-1)}>Fit page</button>
      </div>
      <select aria-label="Paper theme" bind:value={readingState.theme}>{#each Object.keys(readerThemes) as theme}<option>{theme}</option>{/each}</select>
      <button title="Keep original PDF colors; theme surrounds the page" aria-pressed={preserve} onclick={()=>{preserve=!preserve;localStorage.setItem("preserve-colors",String(preserve))}}>Preserve colors</button>
      <button onclick={loadOutline}>Outline</button>
      <button aria-pressed={thumbnails} onclick={()=>thumbnails=!thumbnails}>Thumbnails</button>
      <button onclick={()=>add('bookmark',readingState.page)}>Bookmark</button>
      <button onclick={async()=>{try{const win=getCurrentWindow();await win.setFullscreen(!await win.isFullscreen())}catch(e){error(String(e))}}}>Fullscreen</button>
    </div>
    <form class="document-search" onsubmit={e=>{e.preventDefault();find()}}>
      <input type="search" aria-label="Search PDF" placeholder="Find in PDF" bind:this={searchBox} bind:value={query}>
      <button>{searching?'Searching…':'Find'}</button>
      {#if searching}<button type="button" onclick={()=>{++searchToken;searching=false}}>Stop</button>{/if}
      {#if matches.length}<select aria-label="Search results" onchange={e=>navigate(Number(e.currentTarget.value),true)}><option value="">Matches ({matches.length})</option>{#each matches as hit}<option value={hit.page}>p. {hit.page} · {hit.snippet}</option>{/each}</select>{/if}
    </form>
    {#if showOutline}<div class="outline">{#each outline as item}<button onclick={()=>navigate(item.page,true)}>{item.title}</button>{:else}<span>No outline available.</span>{/each}</div>{/if}
    <div class="reader-body">
    {#if thumbnails}
      <aside class="thumbnail-list" aria-label="Page thumbnails">
        <Rows items={allPages} size={118}>
          {#snippet row(page:number)}<Thumbnail pdf={pdf!} {entry} {page} go={number=>navigate(number,true)}/>{/snippet}
        </Rows>
      </aside>
    {/if}
    <div class="paper-scroll" bind:this={surface} bind:clientWidth={width} bind:clientHeight={height} onscroll={scroll} style:--paper={readerThemes[readingState.theme]}>
      {#if readingState.mode!=='single'}<div style:height={Math.max(0,start-1)*stride+'px'}></div>{/if}
      {#each visible as page (page)}
        <div class="page-slot" style:height={stride+'px'} style:min-width={pageWidth+'px'}>
          <Page {pdf} {page} width={pageWidth} height={pageHeight} theme={readerThemes[readingState.theme]} {preserve} notes={notePages.get(page)??[]} {error} {activate}/>
        </div>
      {/each}
      {#if readingState.mode!=='single'}<div style:height={Math.max(0,pdf.numPages-end)*stride+'px'}></div>{/if}
    </div>
    </div>
  {/if}
  {#if selection}<div class="selection-tools" style:left={selection.left+'px'} style:top={selection.top+'px'}>
    <select aria-label="Annotation color" bind:value={color}>{#each Object.entries(colors) as [name,hex]}<option value={hex}>{name}</option>{/each}</select>
    <button onpointerdown={e=>e.preventDefault()} onclick={()=>saveSelection('highlight')}>Highlight</button>
    <button onpointerdown={e=>e.preventDefault()} onclick={()=>saveSelection('underline')}>Underline</button>
    <button onpointerdown={e=>e.preventDefault()} onclick={()=>saveSelection('selection')}>Add note</button>
    <button onpointerdown={e=>e.preventDefault()} onclick={()=>selection&&navigator.clipboard.writeText(selection.quote).catch(e=>error(String(e)))}>Copy</button>
  </div>{/if}
</div>
