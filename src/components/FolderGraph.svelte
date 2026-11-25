<script lang="ts">
  import { untrack } from 'svelte';
  import { baseName, call, children, type Entry, type Library } from '../model';
  let {library,selected,revision,choose,open,error}: {
    library:Library; selected:Entry|null; revision:number;
    choose:(entry:Entry|null)=>void; open:(entry:Entry)=>void; error:(message:string)=>void
  }=$props();
  const batch=24;
  const palette=['#4c75be','#8062b0','#a25e45','#3d877f','#a05378','#827135'];
  const tone=(depth:number)=>palette[depth%palette.length];
  let ancestors=$state<Entry[]>([]);
  let branch=$state<Entry[]>([]);
  let siblings=$state<Entry[]>([]);
  let related=$state<Entry[]>([]);
  let ready=$state(false);
  let key=$state('');
  let branchLimit=$state(batch);
  let siblingLimit=$state(batch);
  let relatedLimit=$state(batch);
  let depth=$derived(selected && ready ? ancestors.length+1 : 0);
  $effect(()=>{
    revision;
    const item=selected, root=library;
    let cancelled=false;
    const focusKey=root.id+':'+(item?.id??'root');
    if(untrack(()=>key)!==focusKey)ready=false;
    Promise.all([
      item?call<Entry[]>('graph_path',{library:root.id,parent:item.parent}):Promise.resolve([]),
      item&&!item.folder?Promise.resolve([]):children(root.id,item?.path??root.root),
      item?children(root.id,item.parent):Promise.resolve([]),
      item&&!item.folder?call<Entry[]>('related',{id:item.id}):Promise.resolve([]),
    ]).then(([parents,kids,peers,links])=>{
      if(cancelled)return;
      ancestors=parents;
      branch=kids;
      siblings=peers.filter(entry=>entry.id!==item?.id);
      related=links;
      if(key!==focusKey){
        key=focusKey;
        branchLimit=batch;
        siblingLimit=batch;
        relatedLimit=batch;
      }
      ready=true;
    }).catch(reason=>{if(!cancelled)error(String(reason))});
    return ()=>{cancelled=true};
  });
  function select(item:Entry){item.folder?choose(item):open(item)}
</script>

{#snippet node(item:Entry|null, level:number, active=false)}
  <button class="graph-node" class:graph-current={active} style:--depth-color={tone(level)}
    aria-current={active?'true':undefined} title={item?.name??library.root}
    onclick={()=>{if(item)select(item);else choose(null)}}>
    <img class="book-icon" src={item&&!item.folder?'/book.svg':'/folder.svg'} alt="">
    <span class="truncate">{item?.name??baseName(library.root)}{item?.missing?' (missing)':''}</span>
    {#if !item||item.folder}<small>{level===0?'Root':'Level '+level}</small>{/if}
  </button>
{/snippet}

<div class="graph" aria-busy={!ready}>
  {#if ready}
    <div class="graph-path" aria-label="Folder path">
      {#if selected}
        {@render node(null,0)}
        {#each ancestors as parent,index (parent.id)}
          {@render node(parent,index+1)}
        {/each}
      {/if}
      {@render node(selected,depth,true)}
    </div>
    {#if branch.length}
      <details class="graph-group" open style:--depth-color={tone(depth+1)}>
        <summary>In this folder <small>{branch.length}</small></summary>
        <ul>
          {#each branch.slice(0,branchLimit) as item (item.id)}
            <li>{@render node(item,depth+1)}</li>
          {/each}
        </ul>
        {#if branch.length>branchLimit}<button class="graph-more" onclick={()=>branchLimit+=batch}>Show more items</button>{/if}
      </details>
    {:else if !selected||selected.folder}
      <p class="muted graph-empty">No folders or PDFs here yet.</p>
    {/if}
    {#if siblings.length}
      <details class="graph-group graph-siblings" style:--depth-color={tone(depth)}>
        <summary>Also in this folder <small>{siblings.length}</small></summary>
        <ul>{#each siblings.slice(0,siblingLimit) as item (item.id)}<li>{@render node(item,depth)}</li>{/each}</ul>
        {#if siblings.length>siblingLimit}<button class="graph-more" onclick={()=>siblingLimit+=batch}>Show more siblings</button>{/if}
      </details>
    {/if}
    {#if related.length}
      <details class="graph-group graph-related">
        <summary>Related PDFs <small>{related.length}</small></summary>
        <ul>{#each related.slice(0,relatedLimit) as item (item.id)}<li><button title={item.name} onclick={()=>open(item)}><img class="book-icon" src="/book.svg" alt=""> {item.name}</button></li>{/each}</ul>
        {#if related.length>relatedLimit}<button class="graph-more" onclick={()=>relatedLimit+=batch}>Show more related PDFs</button>{/if}
      </details>
    {/if}
  {:else}
    <p class="muted graph-empty" role="status">Loading folder graph…</p>
  {/if}
</div>
