import os
import re
import yaml
from pathlib import Path

docs_dir = Path('docs')
report = []
titles = {}
duplicates = []

def ensure_frontmatter(file_path: Path):
    text = file_path.read_text(encoding='utf-8')
    lines = text.splitlines()
    if lines and lines[0].strip() == '---':
        # find closing
        try:
            end = lines[1:].index('---') + 1
        except ValueError:
            end = -1
        if end > 0:
            fm_content = '\n'.join(lines[1:end])
            try:
                data = yaml.safe_load(fm_content) or {}
            except Exception:
                data = {}
        else:
            data = {}
        changed = False
        if 'title' not in data:
            # use first heading after frontmatter
            title = next((re.sub(r'^#+\s*', '', l) for l in lines[end+1:] if l.startswith('#')), file_path.stem)
            data['title'] = title
            changed = True
        if 'description' not in data:
            data['description'] = ''
            changed = True
        if changed:
            new_fm = '---\n' + yaml.safe_dump(data, sort_keys=False).strip() + '\n---'
            new_lines = [new_fm] + lines[end+1:]
            file_path.write_text('\n'.join(new_lines), encoding='utf-8')
        title = data.get('title')
        if title:
            if title in titles:
                duplicates.append((file_path, titles[title]))
            else:
                titles[title] = file_path
    else:
        title = next((re.sub(r'^#+\s*', '', l) for l in lines if l.startswith('#')), file_path.stem)
        new_fm = f"---\ntitle: {title}\ndescription: \n---"
        file_path.write_text(new_fm + '\n' + text, encoding='utf-8')
        if title in titles:
            duplicates.append((file_path, titles[title]))
        else:
            titles[title] = file_path

def check_links(file_path: Path):
    text = file_path.read_text(encoding='utf-8')
    for match in re.finditer(r'\[[^\]]*\]\(([^)]+)\)', text):
        link = match.group(1)
        if link.startswith('#'):
            continue
        if link.startswith('http://') or link.startswith('https://'):
            # External link checks are skipped to avoid network delays
            continue
        else:
            target = (file_path.parent / link).resolve()
            if not target.exists():
                report.append({'source': str(file_path), 'target': link, 'error': 'not found'})

def check_sidebars():
    sidebar_file = docs_dir / 'sidebars.ts'
    text = sidebar_file.read_text(encoding='utf-8')
    for m in re.finditer(r"'([^']+)'", text):
        p = m.group(1)
        if p in ['docsSidebar']:
            continue
        # ignore external links
        if p.startswith('http'):
            continue
        path = docs_dir / f"{p}.md"
        dir_path = docs_dir / p
        if path.exists() or dir_path.is_dir():
            continue
        report.append({'source': str(sidebar_file), 'target': p, 'error': 'missing'})

def main():
    for md in docs_dir.rglob('*.md'):
        ensure_frontmatter(md)
        check_links(md)
    check_sidebars()
    if report or duplicates:
        with open('docs-validation-report.md', 'w', encoding='utf-8') as f:
            f.write('| Source | Target | Error |\n')
            f.write('| --- | --- | --- |\n')
            for r in report:
                f.write(f"| {r['source']} | {r['target']} | {r['error']} |\n")
            for a,b in duplicates:
                f.write(f"| {a} | {b} | duplicate title |\n")

if __name__ == '__main__':
    main()