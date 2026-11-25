<script lang="ts">
  import { tick } from 'svelte';
  import PaneDivider from './PaneDivider.svelte';
  import { children, baseName, type Library, type Entry } from '../model';
  import Rows from './Rows.svelte';
  import FolderGraph from './FolderGraph.svelte';
  let {library,selected,revision,choose,open,error}: {
    library:Library;selected:Entry|null;revision:number;choose:(entry:Entry|null)=>void;open:(entry:Entry)=>void;error:(message:string)=>void
  } = $props();
  type Column = {path:string;title:string;items:Entry[]};
  type Branch = {entry:Entry;depth:number};
  let view = $state('Tree');
  let trail = $state<Entry[]>([]);
  let columns = $state<Column[]>([]);
  let columnStrip=$state<HTMLDivElement>();
  let stripWidth=$state(0);
  let columnWidths=$state<Record<string,number>>({});
  let revealNext=false;
  let expanded = $state<Record<string,Entry[]>>({});
  let root = $derived(columns[0]?.items ?? []);
  function flatten(entries:Entry[], depth=0):Branch[] {
    return entries.flatMap(entry=>[{entry,depth},...flatten(expanded[entry.path]??[],depth+1)]);
  }
  let tree = $derived(flatten(root));
  $effect(()=>{
    revision;
    const folders = [{path:library.root,title:baseName(library.root)},...trail.map(e=>({path:e.path,title:e.name}))];
    let cancelled=false;
    Promise.all(folders.map(async column=>({...column,items:await children(library.id,column.path)})))
      .then(async data=>{
        if(cancelled)return;
        columns=data;
        if(revealNext){
          revealNext=false;
          await tick();
          if(columnStrip)columnStrip.scrollLeft=columnStrip.scrollWidth;
        }
      }).catch(e=>{if(!cancelled)error(String(e))});
    return ()=>{cancelled=true};
  });
  $effect(()=>{
    const strip=columnStrip, item=selected;
    if(!strip||!stripWidth||!item||item.folder)return;
    const columnIndex=columns.findIndex(column=>column.path===item.parent);
    if(columnIndex<0)return;
    let cancelled=false;
    void tick().then(()=>{
      if(cancelled)return;
      const column=strip.querySelectorAll('.column')[columnIndex];
      if(!column)return;
      const frame=strip.getBoundingClientRect(), bounds=column.getBoundingClientRect();
      if(bounds.left<frame.left||bounds.right>frame.right)strip.scrollLeft+=bounds.left-frame.left;
    });
    return ()=>{cancelled=true};
  });
  function selectColumn(entry:Entry,index:number) {
    if(entry.folder){
      revealNext=true;
      trail=[...trail.slice(0,index),entry];
    }else if(trail.length>index)trail=trail.slice(0,index);
    entry.folder?choose(entry):open(entry);
  }
  async function toggle(entry:Entry) {
    try {
      if(expanded[entry.path])delete expanded[entry.path];
      else expanded[entry.path]=await children(library.id,entry.path);
    } catch(e){error(String(e))}
  }
  function focusSibling(button:HTMLButtonElement, direction:number){
    const buttons=Array.from(button.closest('.virtual')?.querySelectorAll('button[data-node]')??[]);
    const next=buttons[buttons.indexOf(button)+direction] as HTMLButtonElement|undefined;
    next?.focus();
  }
</script>
<div class="library">
  <div class="segmented" aria-label="Library view">
    {#each ['Columns','Tree','Graph'] as mode}
      <button class:active={view===mode} onclick={()=>view=mode}>{mode}</button>
    {/each}
  </div>
  {#if view==='Columns'}
    <div class="columns" bind:this={columnStrip} bind:clientWidth={stripWidth}>
      {#each columns as column,index (column.path)}
        <section class="column" style:flex-basis={(columnWidths[column.path]??195)+"px"}>
          <h3 title={column.path}>{column.title}</h3>
          <Rows items={column.items} height="calc(100% - 38px)">
            {#snippet row(entry:Entry)}
              <button class="entry" class:chosen={selected?.id===entry.id||trail[index]?.id===entry.id} title={entry.name} onclick={()=>selectColumn(entry,index)}>
                <img class="book-icon" src={entry.folder?"/folder.svg":"/book.svg"} alt=""><span class="truncate">{entry.name}{entry.missing?' (missing)':''}</span>{#if entry.folder}<span class="arrow">›</span>{/if}
              </button>
            {/snippet}
          </Rows>
        </section>
        <PaneDivider label={"Resize "+column.title+" column"} value={columnWidths[column.path]??195} min={130} max={430}
          resize={width=>columnWidths[column.path]=width}/>
      {/each}
    </div>
  {:else if view==='Tree'}
    <div class="tree">
      <button class="entry" onclick={()=>choose(null)}><img class="book-icon" src="/folder.svg" alt=""> {baseName(library.root)}</button>
      <Rows items={tree} height="calc(100% - 36px)">
        {#snippet row(branch:Branch)}
          {@const entry=branch.entry}
          <div class="tree-row" style:padding-left={Math.min(14*branch.depth,200)+'px'}>
            {#if entry.folder}<button class="disclosure" aria-label={expanded[entry.path]?'Collapse folder':'Expand folder'} aria-expanded={Boolean(expanded[entry.path])} onclick={()=>toggle(entry)}>{expanded[entry.path]?'▾':'▸'}</button>{:else}<span class="disclosure"></span>{/if}
            <button data-node class="entry" class:chosen={selected?.id===entry.id} title={entry.name} onclick={()=>entry.folder?choose(entry):open(entry)} ondblclick={()=>entry.folder&&toggle(entry)}
              onkeydown={event=>{
                if(event.key==='ArrowRight' && entry.folder && !expanded[entry.path]){event.preventDefault();toggle(entry)}
                if(event.key==='ArrowLeft' && expanded[entry.path]){event.preventDefault();toggle(entry)}
                if(event.key==='ArrowUp'||event.key==='ArrowDown'){event.preventDefault();focusSibling(event.currentTarget,event.key==='ArrowUp'?-1:1)}
              }}>
              <img class="book-icon" src={entry.folder?"/folder.svg":"/book.svg"} alt=""><span class="truncate">{entry.name}{entry.missing?' (missing)':''}</span>
            </button>
          </div>
        {/snippet}
      </Rows>
    </div>
  {:else}
    <FolderGraph {library} {selected} {revision} {choose} {open} {error}/>
  {/if}
</div>
