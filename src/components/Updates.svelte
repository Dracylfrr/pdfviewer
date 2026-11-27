<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { call } from '../model';

  let {show}: {show:boolean}=$props();
  let signed=$state(false), busy=$state(false), configured=$state(false);
  let current=$state(''), message=$state('');
  let automatic=$state(localStorage.getItem('automatic-updates')!=='false');
  let pending=$state.raw<Update|null>(null);

  async function checkForUpdate(){
    if(!signed||busy||pending)return;
    busy=true;message='';
    let candidate:Update|null=null;
    try {
      candidate=await check({timeout:8000});
      if(candidate){
        message='Downloading the verified update…';
        await candidate.download(undefined,{timeout:60000});
        pending=candidate;
        message='';
      }else message='You’re up to date.';
    } catch {
      await candidate?.close().catch(()=>{});
      message='Couldn’t check for or download an update. You can keep reading offline.';
    } finally {
      busy=false;
    }
  }

  onMount(()=>{
    void Promise.all([call<boolean>('updates_enabled'),getVersion()]).then(([enabled,version])=>{
      signed=enabled;current=version;configured=true;
      if(signed&&automatic)void checkForUpdate();
    }).catch(()=>{message='Update settings are unavailable.'});
  });

  export async function applyPending(){
    const update=pending;
    if(!update)return;
    pending=null;
    try {
      await update.install({restartAfterInstall:false});
    } catch(error) {
      console.warn('PDF Viewer could not install the downloaded update.',error);
      await update.close().catch(()=>{});
    }
  }
</script>

{#if show||pending}
  <section class="settings" aria-label="Application updates">
    <span>Version {current}</span>
    {#if show&&signed}
      <button disabled={busy||!configured||Boolean(pending)} onclick={checkForUpdate}>{busy?'Checking…':'Check for updates'}</button>
      <label><input type="checkbox" bind:checked={automatic} onchange={()=>localStorage.setItem('automatic-updates',String(automatic))}> Check at startup</label>
    {:else if show&&configured}
      <span>Automatic updates require the signed Windows installer.</span>
    {/if}
    {#if pending}<span role="status">v{pending.version} will install when you close PDF Viewer.</span>{/if}
    {#if message}<span role="status">{message}</span>{/if}
  </section>
{/if}
