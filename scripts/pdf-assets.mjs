import { cpSync, mkdirSync } from 'node:fs';
mkdirSync('public', { recursive: true });
for (const directory of ['cmaps', 'standard_fonts', 'wasm']) {
  cpSync('node_modules/pdfjs-dist/' + directory, 'public/' + directory, { recursive: true });
}
