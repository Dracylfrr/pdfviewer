<script lang="ts">
  import { baseName, call, type Entry } from '../model';
  import RecentCover from './RecentCover.svelte';
  let {open,addLibrary,hasLibraries}: {open:(item:Entry)=>void;addLibrary:()=>void;hasLibraries:boolean}=$props();
  type Recent={entry:Entry;page:number};
  let recent=$state<Recent[]>([]);
  let busy=$state(true), failure=$state(false), more=$state(false);
  async function load(offset:number){
    busy=true;failure=false;
    try {
      const items=await call<Recent[]>('recent_pdfs',{offset});
      recent=offset?[...recent,...items.slice(0,24)]:items.slice(0,24);
      more=items.length>24;
    } catch {failure=true}
    finally {busy=false}
  }
  let clearFailed=$state(false);
  async function clearRecent(){
    if(busy || !confirm('Clear the recently opened list? Your reading progress and annotations will be kept.'))return;
    busy=true;clearFailed=false;
    try{
      await call('clear_recent_pdfs');
      recent=[];more=false;failure=false;
    }catch{clearFailed=true}
    finally{busy=false}
  }
  $effect(()=>{void load(0)});
</script>
<section class="home" aria-labelledby="home-title">
  <header class="home-header">
    <div><h1 id="home-title">Recently opened</h1><p class="muted">Pick up where you left off.</p></div>
    {#if recent.length}<button disabled={busy} onclick={clearRecent}>Clear recent PDFs</button>{/if}
  </header>
  {#if clearFailed}<p role="alert">Could not clear recent PDFs. Try again.</p>{/if}
  {#if recent.length}
    <div class="recent-grid">
      {#each recent as item (item.entry.id)}
        <button class="recent-card" title={item.entry.name} onclick={()=>open(item.entry)}>
          <RecentCover entry={item.entry}/>
          <span class="recent-caption"><strong class="truncate">{item.entry.name}</strong>
            <small class="truncate">{baseName(item.entry.parent)} · Page {item.page}</small></span>
        </button>
      {/each}
    </div>
  {:else if !busy&&!failure}
    <div class="home-empty"><img class="book-icon" src="/book.svg" alt=""><p>Your recent PDFs will appear here after you open them.</p>
    {#if !hasLibraries}<button class="primary" onclick={addLibrary}>Choose Library Folder</button>{/if}</div>
  {/if}
  {#if busy}<p role="status" class="muted">Loading recent PDFs…</p>{/if}
  {#if failure}<p role="alert">Couldn’t load recent PDFs. <button onclick={()=>load(recent.length)}>Retry</button></p>{/if}
  {#if more&&!busy&&!failure}<button class="recent-more" onclick={()=>load(recent.length)}>Show older PDFs</button>{/if}
</section>
