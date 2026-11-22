<script lang="ts">
  import { children, baseName, call, type Library, type Entry } from '../model';
  import Rows from './Rows.svelte';
  let {library,selected,revision,choose,open,error}: {
    library:Library;selected:Entry|null;revision:number;choose:(entry:Entry|null)=>void;open:(entry:Entry)=>void;error:(message:string)=>void
  } = $props();
  type Column = {path:string;title:string;items:Entry[]};
  type Branch = {entry:Entry;depth:number};
  let view = $state('Columns');
  let trail = $state<Entry[]>([]);
  let columns = $state<Column[]>([]);
  let expanded = $state<Record<string,Entry[]>>({});
  let root = $derived(columns[0]?.items ?? []);
  function flatten(entries:Entry[], depth=0):Branch[] {
    return entries.flatMap(entry=>[{entry,depth},...flatten(expanded[entry.path]??[],depth+1)]);
  }
  let tree = $derived(flatten(root));
  let related = $state<Entry[]>([]);
  let graphChildren = $state<Entry[]>([]);
  let graphSiblings = $state<Entry[]>([]);
  let graphAncestors = $state<Entry[]>([]);
  let graphLimit = $state(24);
  let showRelated = $state(true);
  let showParents = $state(true);
  let showChildren = $state(true);
  $effect(()=>{
    revision;
    const folders = [{path:library.root,title:baseName(library.root)},...trail.map(e=>({path:e.path,title:e.name}))];
    let cancelled=false;
    Promise.all(folders.map(async column=>({...column,items:await children(library.id,column.path)})))
      .then(data=>{if(!cancelled)columns=data}).catch(e=>error(String(e)));
    return ()=>{cancelled=true};
  });
  $effect(()=>{
    if(view!=='Graph')return;
    revision;
    const entry=selected;
    let cancelled=false;
    const parent=entry?.parent??library.root;
    const ancestors:Entry[]=[];
    const load=async()=>{
      let path=parent;
      while(path!==library.root && path.length>library.root.length && ancestors.length<2){
        const ancestorParent=path.slice(0,path.lastIndexOf('\\')===-1?path.lastIndexOf('/'):path.lastIndexOf('\\'));
        const ancestor=(await children(library.id,ancestorParent)).find(e=>e.path===path);
        if(!ancestor)break;
        ancestors.unshift(ancestor); path=ancestorParent;
      }
      const [siblings,kids,links]=await Promise.all([
        entry ? children(library.id,parent) : Promise.resolve([]),
        entry?.folder ? children(library.id,entry.path) : entry ? Promise.resolve([]) : children(library.id,library.root),
        entry && !entry.folder ? call<Entry[]>('related',{id:entry.id}) : Promise.resolve([]),
      ]);
      if(cancelled)return;
      graphSiblings=siblings.filter(e=>e.id!==entry?.id); graphChildren=kids; related=links; graphAncestors=ancestors;
    };
    load().catch(e=>error(String(e)));
    return ()=>{cancelled=true};
  });
  function selectColumn(entry:Entry,index:number) {
    trail=entry.folder?[...trail.slice(0,index),entry]:trail.slice(0,index);
    choose(entry);
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
    <div class="columns">
      {#each columns as column,index (column.path)}
        <section class="column">
          <h3 title={column.path}>{column.title}</h3>
          <Rows items={column.items} height="calc(100% - 38px)">
            {#snippet row(entry:Entry)}
              <button class="entry" class:chosen={selected?.id===entry.id||trail[index]?.id===entry.id} title={entry.name} onclick={()=>selectColumn(entry,index)} ondblclick={()=>!entry.folder&&open(entry)}>
                <span class:folder={entry.folder}>{#if entry.folder}▰{:else}<img class="book-icon" src="/book.svg" alt="">{/if}</span><span class="truncate">{entry.name}{entry.missing?' (missing)':''}</span>{#if entry.folder}<span class="arrow">›</span>{/if}
              </button>
            {/snippet}
          </Rows>
        </section>
      {/each}
    </div>
  {:else if view==='Tree'}
    <div class="tree">
      <button class="entry" onclick={()=>choose(null)}>▰ {baseName(library.root)}</button>
      <Rows items={tree} height="calc(100% - 36px)">
        {#snippet row(branch:Branch)}
          {@const entry=branch.entry}
          <div class="tree-row" style:padding-left={Math.min(14*branch.depth,200)+'px'}>
            {#if entry.folder}<button class="disclosure" aria-label={expanded[entry.path]?'Collapse folder':'Expand folder'} aria-expanded={Boolean(expanded[entry.path])} onclick={()=>toggle(entry)}>{expanded[entry.path]?'▾':'▸'}</button>{:else}<span class="disclosure"></span>{/if}
            <button data-node class="entry" class:chosen={selected?.id===entry.id} title={entry.name} onclick={()=>choose(entry)} ondblclick={()=>entry.folder?toggle(entry):open(entry)}
              onkeydown={event=>{
                if(event.key==='ArrowRight' && entry.folder && !expanded[entry.path]){event.preventDefault();toggle(entry)}
                if(event.key==='ArrowLeft' && expanded[entry.path]){event.preventDefault();toggle(entry)}
                if(event.key==='ArrowUp'||event.key==='ArrowDown'){event.preventDefault();focusSibling(event.currentTarget,event.key==='ArrowUp'?-1:1)}
                if(event.key==='Enter'&&!entry.folder){event.preventDefault();open(entry)}
              }}>
              <span class:folder={entry.folder}>{#if entry.folder}▰{:else}<img class="book-icon" src="/book.svg" alt="">{/if}</span><span class="truncate">{entry.name}{entry.missing?' (missing)':''}</span>
            </button>
          </div>
        {/snippet}
      </Rows>
    </div>
  {:else}
    <div class="graph">
      <div class="graph-controls">
        <button onclick={()=>showParents=!showParents}>{showParents?'Hide':'Expand'} parents</button>
        <button onclick={()=>{showChildren=true;graphLimit+=24}}>Expand children</button>
        <button onclick={()=>showRelated=!showRelated}>{showRelated?'Hide':'Expand'} related</button>
        <button onclick={()=>showChildren=false}>Collapse branch</button>
        <button onclick={()=>{choose(null);graphLimit=24;showChildren=true;showParents=true;showRelated=true}}>Reset</button>
      </div>
      <div class="graph-grid">
        {#if showParents&&selected}
          <div class="nodes">
            <button onclick={()=>choose(null)}>▰ {baseName(library.root)}</button>
            {#each graphAncestors as node (node.id)}<button onclick={()=>choose(node)}>▰ {node.name}</button>{/each}
          </div>
        {/if}
        <div class="nodes">
          <button class="chosen" ondblclick={()=>selected&&!selected.folder&&open(selected)}>{#if selected?.folder}▰{:else}<img class="book-icon" src="/book.svg" alt="">{/if} {selected?.name??baseName(library.root)}</button>
          {#each graphSiblings.slice(0,graphLimit) as node (node.id)}<button onclick={()=>choose(node)}> {#if node.folder}▰{:else}<img class="book-icon" src="/book.svg" alt="">{/if} {node.name}</button>{/each}
        </div>
        {#if showChildren||showRelated}
          <div class="nodes">
            {#if showChildren}{#each graphChildren.slice(0,graphLimit) as node (node.id)}<button onclick={()=>choose(node)}>{#if node.folder}▰{:else}<img class="book-icon" src="/book.svg" alt="">{/if} {node.name}</button>{/each}{/if}
            {#if showRelated}{#each related.slice(0,graphLimit) as node (node.id)}<button onclick={()=>choose(node)}>↔ {node.name}</button>{/each}{/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
