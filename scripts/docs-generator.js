import { promises as fs } from 'fs';
import path from 'path';

const COMPONENTS_DIR = path.join('src', 'components');
const DOCS_DIR = path.join('docs', 'components');
const API_DIR = path.join('docs', 'api');

async function getTsFiles() {
  const files = await fs.readdir(COMPONENTS_DIR);
  return files.filter(f => f.endsWith('.tsx') && !f.includes('.stories') && !f.includes('.demo'));
}

function extractFirstComment(text) {
  const match = text.match(/\/\/\s*(.*)/);
  return match ? match[1].trim() : '';
}

function extractProps(text) {
  const result = [];
  const match = text.match(/interface\s+\w*Props\s*{([\s\S]*?)}/);
  if (!match) return result;
  const body = match[1];
  const propRegex = /\s*(\w+)\??:\s*([^;]+)/g;
  let m;
  while ((m = propRegex.exec(body)) !== null) {
    result.push({ name: m[1], type: m[2].trim() });
  }
  return result;
}

function extractHooks(text) {
  const hooks = ['useState','useEffect','useRef','useCallback','useMemo'];
  return hooks.filter(h => text.includes(h));
}

async function extractTauriCommands() {
  const mainFile = path.join('src-tauri', 'src', 'main.rs');
  const text = await fs.readFile(mainFile, 'utf8');
  const regex = /#\[tauri::command\]\s*fn\s+(\w+)\s*\(([^)]*)\)\s*(->\s*[^\s{]+)?/g;
  const commands = [];
  let m;
  while ((m = regex.exec(text)) !== null) {
    commands.push({
      name: m[1],
      params: m[2].replace(/\s+/g, ' ').trim(),
      returns: m[3] ? m[3].replace('->', '').trim() : ''
    });
  }
  return commands;
}

async function generateApiDocs() {
  await fs.mkdir(API_DIR, { recursive: true });
  const cmds = await extractTauriCommands();
  let md = `---\ntitle: IPC Interface\ndescription: Auto generated list of Tauri commands\n---\n\n`;
  md += '## Commands\n\n| Name | Parameters | Returns |\n| --- | --- | --- |\n';
  for (const c of cmds) {
    md += `| \`${c.name}\` | \`${c.params}\` | \`${c.returns}\` |\n`;
  }
  await fs.writeFile(path.join(API_DIR, 'ipc-interface.md'), md, 'utf8');

  const indexMd = `---\ntitle: API Übersicht\ndescription: Generated API reference\n---\n\n- [IPC Interface](ipc-interface.md)\n`;
  await fs.writeFile(path.join(API_DIR, 'index.md'), indexMd, 'utf8');
}

const featureLinks = {
  ClipboardSync: '../features/clipboard.md',
  ConnectionManager: '../features/remote.md',
  RemoteScreen: '../features/remote.md',
  FileTransfer: '../features/files.md',
};

function createDoc(name, desc, props, hooks, sourcePath) {
  let md = `---\ntitle: ${name}\ndescription: ${desc}\n---\n\n`;
  md += `## Function\n${desc || 'TBD'}\n\n`;
  if (props.length) {
    md += '## Props\n\n| Name | Type |\n| --- | --- |\n';
    for (const p of props) {
      md += `| \`${p.name}\` | \`${p.type}\` |\n`;
    }
  }
  if (hooks.length) {
    md += '\n## Used Hooks\n\n';
    md += hooks.map(h => `\`${h}\``).join(', ');
    md += '\n';
  }
  if (featureLinks[name]) {
    md += `\n## Related Features\n\n- [Feature Documentation](${featureLinks[name]})\n`;
  }
  md += `\n## Source\n\n[${sourcePath}](/${sourcePath})\n`;
  return md;
}

async function main() {
  await fs.mkdir(DOCS_DIR, { recursive: true });
  const tsFiles = await getTsFiles();
  const indexEntries = [];

  // Process React components
  for (const file of tsFiles) {
    const filePath = path.join(COMPONENTS_DIR, file);
    const text = await fs.readFile(filePath, 'utf8');
    const name = path.basename(file, '.tsx');
    const desc = extractFirstComment(text);
    const props = extractProps(text);
    const hooks = extractHooks(text);
    const doc = createDoc(name, desc, props, hooks, filePath);
    await fs.writeFile(path.join(DOCS_DIR, `${name}.md`), doc, 'utf8');
    indexEntries.push({ name, desc });
  }

  // Process Rust modules
  const rustRoot = path.join('src-tauri', 'src');
  const rustModules = await fs.readdir(rustRoot);
  for (const mod of rustModules) {
    const modFile = path.join(rustRoot, mod, 'mod.rs');
    try {
      const text = await fs.readFile(modFile, 'utf8');
      const descMatch = text.match(/\/\/\s*(.*)/);
      const desc = descMatch ? descMatch[1].trim() : '';
      const structs = Array.from(text.matchAll(/pub struct (\w+)/g)).map(m => m[1]);
      const fns = Array.from(text.matchAll(/pub fn (\w+)/g)).map(m => m[1]);
      let md = `---\ntitle: ${mod}\ndescription: ${desc}\n---\n\n`;
      if (structs.length) {
        md += '## Public Structs\n';
        for (const s of structs) md += `- \`${s}\`\n`;
      }
      if (fns.length) {
        md += '\n## Public Functions\n';
        for (const f of fns) md += `- \`${f}\`\n`;
      }
      md += `\n## Source\n\n[${modFile}](/${modFile})\n`;
      await fs.writeFile(path.join(DOCS_DIR, `${mod}.md`), md, 'utf8');
      indexEntries.push({ name: mod, desc });
    } catch (err) {
      // ignore missing mod.rs
    }
  }

  let indexMd = `---\ntitle: Components Overview\ndescription: Reference of all UI components\n---\n\n`;
  for (const entry of indexEntries) {
    indexMd += `- [${entry.name}](${entry.name}.md) - ${entry.desc}\n`;
  }
  await fs.writeFile(path.join(DOCS_DIR, 'index.md'), indexMd, 'utf8');

  // Generate API documentation
  await generateApiDocs();
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
