<script lang="ts">
  import type { Snippet } from 'svelte';
  let {items, row, height = '100%', size = 36}: {items:unknown[];row:Snippet<[any]>;height?:string;size?:number} = $props();
  let top = $state(0);
  let viewport = $state(0);
  let start = $derived(Math.min(Math.max(0, Math.floor(top / size) - 5),Math.max(0,items.length-1)));
  let end = $derived(Math.min(items.length,start + Math.ceil(viewport / size) + 12));
</script>
<div class="virtual" style:height bind:clientHeight={viewport} onscroll={e=>top=e.currentTarget.scrollTop}>
  <div style:height={start*size+'px'}></div>
  {#each items.slice(start,end) as item}
    <div class="virtual-row" style:height={size+'px'}>{@render row(item)}</div>
  {/each}
  <div style:height={(items.length-end)*size+'px'}></div>
</div>
