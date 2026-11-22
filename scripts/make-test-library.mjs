import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
const root=process.argv[2];
if(!root)throw new Error('Pass the destination folder for the generated test library.');
function pdf(path,pageCount,caption){
  const objects=[
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Count '+pageCount+' /Kids ['+Array.from({length:pageCount},(_,i)=>(4+i*2)+' 0 R').join(' ')+'] >>',
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>'
  ];
  for(let i=0;i<pageCount;i++){
    const landscape=i%3===2, rotation=i%4===3?90:0;
    const page=objects.length+1;
    const stream='BT /F1 22 Tf 56 415 Td ('+caption+' - page '+(i+1)+') Tj 0 -34 Td /F1 14 Tf (Highlight this sentence to test PDF annotations.) Tj ET';
    objects.push('<< /Type /Page /Parent 2 0 R /MediaBox [0 0 '+(landscape?'700 480':'500 700')+'] /Rotate '+rotation+
      ' /Resources << /Font << /F1 3 0 R >> >> /Contents '+(page+1)+' 0 R >>',
      '<< /Length '+stream.length+' >>\nstream\n'+stream+'\nendstream');
  }
  let body='%PDF-1.7\n';
  const offsets=['0000000000 65535 f \n'];
  for(let i=0;i<objects.length;i++){
    offsets.push(String(body.length).padStart(10,'0')+' 00000 n \n');
    body+=(i+1)+' 0 obj\n'+objects[i]+'\nendobj\n';
  }
  const xref=body.length;
  body+='xref\n0 '+offsets.length+'\n'+offsets.join('')+'trailer\n<< /Size '+offsets.length+' /Root 1 0 R >>\nstartxref\n'+xref+'\n%%EOF\n';
  writeFileSync(path,body,{flag:'wx'});
}
const science=join(root,'Computer Science','Algorithms');
const math=join(root,'Mathematics');
mkdirSync(science,{recursive:true}); mkdirSync(math,{recursive:true});
pdf(join(science,'Algorithms.pdf'),4,'Algorithms');
pdf(join(science,'Data Structures.pdf'),1,'Data Structures');
pdf(join(math,'Calculus.pdf'),1250,'Calculus');
writeFileSync(join(math,'Corrupt.pdf'),'Broken fixture',{flag:'wx'});
