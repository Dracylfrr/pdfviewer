import { writeFileSync, readFileSync } from 'node:fs';
const repo=process.env.GITHUB_REPOSITORY;
const key=process.env.PDFVIEWER_UPDATER_PUBLIC_KEY?.trim();
if(!/^[\w.-]+\/[\w.-]+$/.test(repo??'')||!key)throw new Error('Set GITHUB_REPOSITORY and PDFVIEWER_UPDATER_PUBLIC_KEY.');
const cargo=readFileSync('src-tauri/Cargo.toml','utf8');
const version=/^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1];
if(process.env.GITHUB_REF_NAME!=='v'+version)throw new Error('The tag must match Cargo.toml.');
writeFileSync('src-tauri/release-config.json',JSON.stringify({
  bundle:{createUpdaterArtifacts:true},
  plugins:{updater:{pubkey:key,windows:{installMode:"quiet"},endpoints:[`https://github.com/${repo}/releases/latest/download/latest.json`]}}
},null,2));
