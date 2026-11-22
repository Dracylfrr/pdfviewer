<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { baseName, call, children, colors, kinds, type Library as LibraryModel, type Entry, type Hit, type Note, type Rect } from './model';
  import Library from './components/Library.svelte';

  import Rows from './components/Rows.svelte';
  import Updates from './components/Updates.svelte';
  let libraries=$state<LibraryModel[]>([]);
  let libraryId=$state(Number(localStorage.getItem('library')||0));
  let library=$derived(libraries.find(item=>item.id===libraryId));
  let selected=$state<Entry|null>(null), document=$state<Entry|null>(null);
  let revision=$state(0);
  let status=$state(''), failure=$state('');
  let notes=$state<Note[]>([]), tags=$state<string[]>([]), related=$state<Entry[]>([]);
  let count=$state({folders:0,pdfs:0});
  let request=$state<{page:number;token:number}|null>(null);
  let focused=$state('');
  let filter=$state('All'), sort=$state('Page');
  let query=$state(''), hits=$state<Hit[]>([]);
  let searchBox:HTMLInputElement;
  let searching=$state(false), searchVersion=0, contextVersion=0, token=0;
  let left=$state(true), right=$state(true), settings=$state(false);
  let chrome=$state(localStorage.getItem('chrome-preference')||'Pastel Blue');
  let saving=$state(0);
  let indexJob:AbortController|undefined;
  let indexingText=$state(false);
  let pending=new Map<string,{timer:ReturnType<typeof setTimeout>;note:Note}>();
  let writes=Promise.resolve();
  let linkSource=$state<Entry|null>(null);
  let tagInput=$state('');
  let context=$derived(document??selected);
  let filtered=$derived(notes.filter(n=>filter==='All'||(filter==='Notes'?['document','selection','folder'].includes(n.kind):n.kind===filter))
    .sort((a,b)=>sort==='Page'?(a.page??0)-(b.page??0):sort==='Color'?a.color.localeCompare(b.color):sort==='Newest'?notes.indexOf(b)-notes.indexOf(a):notes.indexOf(a)-notes.indexOf(b)));
  function error(message:string){failure=message}
  async function attempt(job:()=>Promise<void>){try{await job()}catch(e){error(String(e))}}
  async function loadLibraries(){
    libraries=await call<LibraryModel[]>('get_libraries');
    if(!libraries.some(l=>l.id===libraryId))libraryId=libraries[0]?.id??0;
  }
  onMount(()=>{
    let disposed=false;
    const events=Promise.all([
      listen<[number,number,boolean]>('library-index',({payload:[id,total,done]})=>{
        if(id!==libraryId)return;
        status=done?'':`Indexing library · ${total} PDFs`;
        revision++;
      }),
      listen<string>('index-error',e=>{status='';error(e.payload)}),
      getCurrentWindow().onCloseRequested(async()=>{await flushAll()}),
    ]);
    events.catch(e=>{if(!disposed)error(String(e))});
    attempt(loadLibraries);
    return ()=>{disposed=true;void events.then(callbacks=>callbacks.forEach(unlisten=>unlisten())).catch(()=>{});void flushAll()};
  });
  $effect(()=>localStorage.setItem('library',String(libraryId)));
  $effect(()=>localStorage.setItem('chrome-preference',chrome));
  $effect(()=>{
    const item=context, version=++contextVersion;
    notes=[];tags=[];related=[];count={folders:0,pdfs:0};
    if(!item)return;
    attempt(async()=>{
      const [loaded,labels,links,items]=await Promise.all([
        call<Note[]>('notes',{id:item.id}), call<string[]>('tags',{id:item.id}),
        item.folder?Promise.resolve([]):call<Entry[]>('related',{id:item.id}),
        item.folder?children(item.library,item.path):Promise.resolve([]),
      ]);
      if(version!==contextVersion)return;
      notes=loaded;tags=labels;related=links;
      count={folders:items.filter(e=>e.folder).length,pdfs:items.filter(e=>!e.folder&&!e.missing).length};
    });
  });
  async function addLibrary(){
    const path=await open({directory:true,title:'Choose Library Folder'});
    if(typeof path!=='string')return;
    status='Indexing library…';
    try{const id=await call<number>('add_library',{path});await loadLibraries();await changeLibrary(id)}
    catch(e){status='';throw e}
  }
  async function changeLibrary(id:number){
    indexJob?.abort();
    await flushAll();libraryId=id;selected=null;document=null;request=null;revision++;
  }
  async function choose(item:Entry|null){await flushAll(); selected=item; document=null; request=null; focused='';}
  async function openDocument(item:Entry,page?:number|null){
    if(item.folder){await choose(item);return}
    if(item.missing){await choose(item);error('This PDF is missing. Its notes are still in the library.');return}
    await flushAll();
    libraryId=item.library; selected=item; document=item; focused='';
    request=page?{page,token:++token}:null;
  }
  async function search(){
    const version=++searchVersion;
    if(!query.trim()){hits=[];return}
    searching=true;
    try{const result=await call<Hit[]>('search',{term:query});if(version===searchVersion)hits=result}
    finally{if(version===searchVersion)searching=false}
  }
  function commit(note:Note){
    saving++;
    writes=writes.then(()=>call('save_note',{note})).then(()=>{},e=>error('Could not save the annotation. '+String(e))).finally(()=>saving--);
    return writes;
  }
  async function flush(id:string){
    const task=pending.get(id);
    if(task){clearTimeout(task.timer);pending.delete(id);await commit(task.note)}
  }
  async function flushAll(){for(const id of pending.keys())void flush(id);await writes;}
  function editNote(note:Note,content:string){
    const updated={...note,content};
    notes=notes.map(n=>n.id===note.id?updated:n);
    clearTimeout(pending.get(note.id)?.timer);
    pending.set(note.id,{note:updated,timer:setTimeout(()=>void flush(note.id),600)});
  }
  function addNote(kind:string,page:number|null,quote='',color:string=colors.Yellow,rects:Rect[]=[]){
    if(!context)return;
    const note:Note={id:crypto.randomUUID(),entry:context.id,kind,page,quote,content:'',color,rects:JSON.stringify(rects)};
    notes=[...notes,note];void commit(note);activate(note.id);
  }
  function activate(id:string){
    right=true;filter='All';focused=id;
    void tick().then(()=>window.document.getElementById('note-'+id)?.scrollIntoView({block:'nearest'}));
  }
  async function removeNote(note:Note){
    await flush(note.id);await call('delete_note',{id:note.id});notes=notes.filter(n=>n.id!==note.id);
  }
  async function changeTag(label:string,remove=false){
    const id=context?.id;if(!id||!label.trim())return;
    await call('set_tag',{id,label,remove});
    const updated=await call<string[]>('tags',{id});if(context?.id===id)tags=updated;
    tagInput='';
  }
  async function linkSelection(){
    if(!context||context.folder)return;
    if(!linkSource){linkSource=context;return}
    const id=context.id;
    await call('link',{a:linkSource.id,b:id});linkSource=null;revision++;
    const updated=await call<Entry[]>('related',{id});
    if(context?.id===id)related=updated;
  }
  async function exportNotes(format:string){
    if(!context)return;
    const path=await save({defaultPath:format==='json'?'annotations.json':'notes.md'});
    if(!path)return;
    const content=format==='json'?JSON.stringify({version:1,document:{name:context.name,fingerprint:context.fingerprint,path:context.path},notes},null,2):
      '# '+context.name+'\n\n'+notes.map(n=>`## ${kinds[n.kind]}${n.page?' · Page '+n.page:''}\n\n${n.quote?'> '+n.quote.replaceAll('\n','\n> ')+'\n\n':''}${n.content}\n`).join('\n');
    await call('write_export',{path,content});
  }
  async function importNotes(){
    const item=context;
    if(!item)return;
    const path=await open({multiple:false,filters:[{name:"Annotation JSON",extensions:["json"]}]});
    if(typeof path!=="string")return;
    await flushAll();
    const args={id:item.id,path,confirmMismatch:false};
    let added=await call<number|null>("import_annotations",args);
    if(added===null&&confirm("This export belongs to a different document. Apply its annotations here?")){
      added=await call<number>("import_annotations",{...args,confirmMismatch:true});
    }
    if(added===null)return;
    const updated=await call<Note[]>("notes",{id:item.id});
    if(context?.id===item.id)notes=updated;
    status=added+" annotations imported";
  }
  async function toggleTextIndex(){
    if(indexJob){indexJob.abort();return}
    if(!library)return;
    const id=library.id;
    const controller=new AbortController();
    indexJob=controller;indexingText=true;
    try{
      const {indexLibrary}=await import("./index-pdf");
      const result=await indexLibrary(id,controller.signal,message=>status=message);
      status=controller.signal.aborted?"PDF text indexing paused":`"Indexed ${result.indexed} PDFs · ${result.skipped} skipped"`;
    }finally{indexingText=false;indexJob=undefined}
  }
  async function locate(item:Entry){
    const path=await open({multiple:false,filters:[{name:"PDF",extensions:["pdf"]}]});
    if(typeof path!=="string")return;
    const args={id:item.id,path,confirmMismatch:false};
    let located=await call<boolean>("locate_pdf",args);
    if(!located&&confirm("This file differs from the original. Relink the missing PDF and its annotations to it?")){
      located=await call<boolean>("locate_pdf",{...args,confirmMismatch:true});
    }
    if(located){revision++;await choose(await call<Entry>("get_entry",{id:item.id}));}
  }
  async function forget(item:Entry){
    if(!confirm("Forget this missing PDF and delete its saved annotations, tags, and reading history?"))return;
    await flushAll();await call("forget_pdf",{id:item.id});revision++;await choose(null);
  }
  async function backup(){
    const path=await save({defaultPath:'pdfviewer-backup.sqlite'});
    if(path){await flushAll();await call('backup',{path});status='Metadata backup saved';}
  }
  async function restore(){
    const path=await open({multiple:false,filters:[{name:'SQLite backup',extensions:['sqlite']}]});
    if(typeof path==='string'&&confirm('Restore this metadata snapshot? The current database will be backed up first. Restart the app after restoring.')){
      await flushAll();await call('restore',{path});await loadLibraries();await changeLibrary(libraryId);status='Backup restored. Restart PDF Viewer to refresh filesystem watching.';
    }
  }
  function keyboard(event:KeyboardEvent){
    if(event.ctrlKey&&event.key.toLowerCase()==='f'&&!document){event.preventDefault();searchBox?.focus()}
    if(event.key==='Escape'){settings=false;hits=[];query=''}
  }
</script>
<svelte:window onkeydown={keyboard}/>
<div class="app" data-theme={chrome}>
  <header class="topbar">
    <div class="brand"><img class="book-icon" src="/book.svg" alt=""> PDF Viewer</div>
    {#if libraries.length}
      <select aria-label="Library" value={libraryId} onchange={e=>changeLibrary(Number(e.currentTarget.value))}>
        {#each libraries as item (item.id)}<option value={item.id} title={item.root}>{baseName(item.root)}</option>{/each}
      </select>
    {/if}
    <button onclick={()=>attempt(addLibrary)}>{libraries.length?'Add library':'Choose Library Folder'}</button>
    <form class="library-search" onsubmit={e=>{e.preventDefault();attempt(search)}}>
      <input type="search" placeholder="Search library" aria-label="Search library" bind:value={query} bind:this={searchBox} oninput={()=>{++searchVersion;searching=false;hits=[]}}>
      <button>{searching?'Searching…':'Search'}</button>
    </form>
    <button aria-pressed={left} onclick={()=>left=!left}>Library</button>
    <button aria-pressed={right} onclick={()=>right=!right}>Notes</button>
    <button aria-expanded={settings} onclick={()=>settings=!settings}>Settings</button>
  </header>
  {#if failure}<div class="banner" role="alert"><span>{failure}</span><button onclick={()=>failure=''}>Dismiss</button></div>{/if}
  {#if status}<div class="status" role="status">{status}</div>{/if}
  {#if settings}
    <section class="settings">
      <label>Application theme <select bind:value={chrome}>{#each ['System','Light','Dark','Pastel Blue','Pastel Lavender','Pastel Sage','Pastel Peach'] as theme}<option>{theme}</option>{/each}</select></label>
      <button onclick={()=>attempt(async()=>{await call("clear_thumbnails");status="Thumbnail cache cleared"})}>Clear thumbnail cache</button>
      {#if library}<button onclick={()=>attempt(async()=>{hits=await call<Hit[]>("missing_entries",{library:libraryId});status=hits.length?"":"No missing PDFs"})}>Find missing PDFs</button>{/if}
      <button onclick={()=>attempt(backup)}>Back up metadata</button>
      <button onclick={()=>attempt(restore)}>Restore backup</button>
      {#if library}<button onclick={()=>attempt(toggleTextIndex)}>{indexingText?"Pause PDF text indexing":"Index PDF text"}</button>{/if}
      {#if library}<button onclick={()=>attempt(async()=>{if(confirm('Remove this library from PDF Viewer? Its files will stay in place.')){await call('remove_library',{id:libraryId});await loadLibraries();await changeLibrary(libraryId)}})}>Remove library from app</button>{/if}
    </section>
  {/if}
  <Updates show={settings} flush={flushAll}/>
  {#if hits.length}
    <div class="results">
      <div class="results-title"><strong>Search results</strong><button onclick={()=>hits=[]}>Close</button></div>
      <Rows items={hits} height="230px" size={54}>
        {#snippet row(hit:Hit)}<button class="result" onclick={()=>{hits=[];attempt(()=>openDocument(hit.entry,hit.page))}}><strong>{hit.entry.name}{hit.page?' · Page '+hit.page:''}</strong><span>{baseName(hit.entry.parent)} · {hit.snippet}</span></button>{/snippet}
      </Rows>
    </div>
  {/if}
  {#if linkSource}<div class="status">Select another PDF to link to {linkSource.name}. <button onclick={()=>linkSource=null}>Cancel</button></div>{/if}
  <main class="workbench" class:has-document={Boolean(document)}>
    {#if library&&left}
      <aside class="navigator" aria-label="Library navigation">
        {#key library.id}<Library {library} {selected} {revision} choose={item=>attempt(()=>choose(item))} open={item=>attempt(()=>openDocument(item))} {error}/>{/key}
      </aside>
    {/if}
    {#if document}
      {#key document.id}{#await import('./components/Reader.svelte') then module}<module.default entry={document} {notes} requested={request} add={addNote} {activate} {error}/>{:catch reason}<div class='empty'>{String(reason)}</div>{/await}{/key}
    {:else if selected}
      <section class="details">
        <span class="detail-icon" aria-hidden="true">{#if selected.folder}▰{:else}<img class="book-icon" src="/book.svg" alt="">{/if}</span>
        <h1>{selected.name}</h1>
        <p class="muted">{baseName(selected.parent)}</p>
        {#if selected.folder}
          <p>{count.folders} folders · {count.pdfs} PDFs</p>
          <button onclick={()=>{right=true;addNote('folder',null)}}>Add folder note</button>
        {:else}
          <p>{(selected.size/1024).toLocaleString(undefined,{maximumFractionDigits:1})} KB · {new Date(selected.modified).toLocaleDateString()}</p>
          <p>{notes.filter(n=>n.kind==='highlight').length} highlights · {notes.filter(n=>n.kind==='bookmark').length} bookmarks</p>
          <button class="primary" disabled={selected.missing} onclick={()=>selected&&attempt(()=>openDocument(selected!))}>{selected.missing?'PDF missing':'Open PDF'}</button>
          {#if selected.missing}
            <button onclick={()=>selected&&attempt(()=>locate(selected!))}>Locate file</button>
            <button onclick={()=>selected&&attempt(()=>forget(selected!))}>Forget missing PDF</button>
          {/if}
          <button onclick={()=>attempt(linkSelection)}>{linkSource?'Link to selected PDF':'Add relationship'}</button>
          {#each related as item (item.id)}<button onclick={()=>attempt(()=>choose(item))}>↔ {item.name}</button>{/each}
          <form class="tags" onsubmit={e=>{e.preventDefault();attempt(()=>changeTag(tagInput))}}>
            {#each tags as tag}<button type="button" title={'Remove tag '+tag} onclick={()=>attempt(()=>changeTag(tag,true))}>{tag} ×</button>{/each}
            <input aria-label="New tag" placeholder="New tag" bind:value={tagInput}><button>Add tag</button>
          </form>
        {/if}
      </section>
    {:else}
      <section class="empty"><span class="detail-icon" aria-hidden="true"><img class="book-icon" src="/book.svg" alt=""></span><h1>{library?'Your PDF library':'Open your PDF library'}</h1>
        <p>{library?'Select a folder or PDF to get started.':'Choose an existing folder to browse and read its PDFs.'}</p>
        {#if !library}<button class="primary" onclick={()=>attempt(addLibrary)}>Choose Library Folder</button>{/if}
      </section>
    {/if}
    {#if right&&context}
      <aside class="notes" aria-label="Annotations">
        <div class="notes-header"><h2>{context.folder?'Folder notes':'Annotations'}</h2><small role="status">{saving?'Saving…':'Saved'}</small></div>
        <div class="note-actions">
          <button onclick={()=>addNote(context?.folder?'folder':'document',null)}>Add note</button>
          <button onclick={()=>attempt(()=>exportNotes('json'))}>Export JSON</button>
          <button onclick={()=>attempt(()=>exportNotes('md'))}>Export Markdown</button>
        </div>
        <div class="note-actions">
          <button onclick={()=>attempt(importNotes)}>Import JSON</button>
          <select aria-label="Filter annotations" bind:value={filter}><option>All</option><option>Notes</option><option value="highlight">Highlights</option><option value="underline">Underlines</option><option value="bookmark">Bookmarks</option></select>
          <select aria-label="Sort annotations" bind:value={sort}>{#each ['Page','Newest','Oldest','Color'] as order}<option>{order}</option>{/each}</select>
        </div>
        {#if filtered.length}
          <Rows items={filtered} size={148}>
            {#snippet row(note:Note)}
              <article id={'note-'+note.id} class:focused={focused===note.id} style:--mark={note.color}>
                <div class="note-card-title">
                  {#if note.page}<button onclick={()=>context&&attempt(()=>openDocument(context!,note.page))}>{kinds[note.kind]} · p. {note.page}</button>{:else}<strong>{kinds[note.kind]}</strong>{/if}
                  <button aria-label="Delete annotation" onclick={()=>attempt(()=>removeNote(note))}>×</button>
                </div>
                {#if note.quote}<blockquote title={note.quote}>{note.quote}</blockquote>{/if}
                <textarea aria-label={kinds[note.kind]+' text'} placeholder="Write a note…" value={note.content} oninput={e=>editNote(note,e.currentTarget.value)} onblur={()=>void flush(note.id)}></textarea>
              </article>
            {/snippet}
          </Rows>
        {:else}<p class="muted">No annotations here yet.</p>{/if}
      </aside>
    {/if}
  </main>
</div>
