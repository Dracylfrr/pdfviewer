import { loadPdf } from './pdf';
import { call } from './model';

type Candidate={id:number;name:string;modified:number;size:number};
export async function indexLibrary(library:number, signal:AbortSignal, progress:(message:string)=>void){
  let after=0, indexed=0, skipped=0;
  while(!signal.aborted){
    const item=await call<Candidate|null>('next_pdf',{library,after});
    if(!item)break;
    after=item.id;
    let task:Awaited<ReturnType<typeof loadPdf>>|undefined;
    try {
      progress(`Indexing PDF text · ${item.name}`);
      task=await loadPdf(item.id);
      if(signal.aborted)break;
      const abort=()=>{void task?.destroy().catch(()=>{})};
      signal.addEventListener('abort',abort,{once:true});
      try {
        const doc=await task.promise;
        let pages:{page:number;text:string}[]=[];
        for(let page=1;page<=doc.numPages&&!signal.aborted;page++){
          const pdfPage=await doc.getPage(page);
          const content=await pdfPage.getTextContent();
          const text=content.items.map(item=>'str' in item?item.str:'').join(' ').slice(0,500_000);
          pages.push({page,text});
          pdfPage.cleanup();
          if(pages.length>=8||page===doc.numPages){
            const valid=await call<boolean>('index_pages',{item,pages,total:doc.numPages,start:page===pages.length,complete:page===doc.numPages});
            pages=[];
            if(!valid)throw new Error('The file changed during indexing.');
            await new Promise(resolve=>setTimeout(resolve,20));
          }
        }
        if(!signal.aborted)indexed++;
      } finally { signal.removeEventListener('abort',abort); }
    }catch{if(!signal.aborted)skipped++}
    finally{await task?.destroy().catch(()=>{})}
  }
  return {indexed,skipped};
}
