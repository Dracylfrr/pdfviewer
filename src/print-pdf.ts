import type { PDFDocumentProxy } from 'pdfjs-dist';

export async function printPdf(pdf:PDFDocumentProxy, first:number, last:number) {
  const output=document.createElement('div');
  output.className='print-output';
  const urls:string[]=[];
  function clean(){
    output.remove();
    for(const url of urls)URL.revokeObjectURL(url);
  }
  try {
    for(let number=first;number<=last;number++){
      const page=await pdf.getPage(number);
      const viewport=page.getViewport({scale:2});
      const canvas=document.createElement('canvas');
      const scale=Math.min(1,Math.sqrt(5_000_000/(viewport.width*viewport.height)));
      const size=page.getViewport({scale:2*scale});
      canvas.width=Math.ceil(size.width);
      canvas.height=Math.ceil(size.height);
      await page.render({canvas,viewport:size,intent:'print',background:'#ffffff'}).promise;
      const bitmap=await new Promise<Blob|null>(resolve=>canvas.toBlob(resolve,'image/png'));
      canvas.width=canvas.height=0;
      if(!bitmap)throw new Error('A page could not be prepared for printing.');
      const url=URL.createObjectURL(bitmap);
      urls.push(url);
      const sheet=document.createElement('div');
      sheet.className='print-sheet';
      const img=new Image();
      img.src=url;
      await img.decode();
      sheet.append(img);
      output.append(sheet);
    }
    document.body.append(output);
    window.addEventListener('afterprint',clean,{once:true});
    window.print();
  } catch(error) {
    window.removeEventListener('afterprint',clean);
    clean();
    throw error;
  }
}
