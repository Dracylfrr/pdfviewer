<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { call } from '../model';
  let {show,flush}: {show:boolean;flush:()=>Promise<void>}=$props();
  let enabled=$state(false), busy=$state(false);
  let current=$state(''), message=$state('');
  let automatic=$state(localStorage.getItem('automatic-updates')==='true');
  let update=$state.raw<Update|null>(null);
  async function checkForUpdate(){
    busy=true;message='';
    try{
      await update?.close();
      update=await check({timeout:15000});
      if(!update)message='You have the latest release.';
    }catch{message='Couldn’t check for updates. You can keep reading offline.'}
    finally{busy=false}
  }
  onMount(()=>{
    void Promise.all([call<boolean>('updates_enabled'),getVersion()]).then(([configured,version])=>{
      enabled=configured;current=version;
      if(enabled&&automatic)void checkForUpdate();
    }).catch(()=>{message='Update settings are unavailable.'});
  });
  async function install(){
    if(!update)return;
    busy=true;message='Downloading and verifying the update…';
    try{await flush();await update.downloadAndInstall()}
    catch{message='Couldn’t install the update. Try checking for updates again.'}
    finally{busy=false}
  }
</script>
{#if show||(update&&!busy)}
  <section class="settings" aria-label="Application updates">
    <span>Version {current}</span>
    {#if enabled}
      <button disabled={busy} onclick={checkForUpdate}>Check for updates</button>
      <label><input type="checkbox" bind:checked={automatic} onchange={()=>localStorage.setItem('automatic-updates',String(automatic))}> Check at startup</label>
      {#if update}
        <button disabled={busy} onclick={install}>Install v{update.version}</button>
        {#if update.body}<details><summary>Release notes</summary><p>{update.body}</p></details>{/if}
      {/if}
    {:else}<span>This build has no release feed.</span>{/if}
    {#if message}<span role="status">{message}</span>{/if}
  </section>
{/if}
