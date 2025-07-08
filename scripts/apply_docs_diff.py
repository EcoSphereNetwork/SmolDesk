#!/usr/bin/env python3
"""
Script to apply the documentation frontmatter changes from the git diff.
This script adds frontmatter to markdown files and creates the validation script.
"""

import os
import re
import yaml
import subprocess
from pathlib import Path
from typing import Dict, List, Tuple

def create_markdownlint_config():
    """Create .markdownlint.yaml configuration file."""
    config = {
        'default': True,
        'MD013': False,
        'MD033': False
    }
    
    with open('.markdownlint.yaml', 'w') as f:
        yaml.dump(config, f, default_flow_style=False)
    
    print("✓ Created .markdownlint.yaml")

def create_docs_validation_script():
    """Create the docs validation Python script."""
    script_content = '''import os
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
            fm_content = '\\n'.join(lines[1:end])
            try:
                data = yaml.safe_load(fm_content) or {}
            except Exception:
                data = {}
        else:
            data = {}
        changed = False
        if 'title' not in data:
            # use first heading after frontmatter
            title = next((re.sub(r'^#+\\s*', '', l) for l in lines[end+1:] if l.startswith('#')), file_path.stem)
            data['title'] = title
            changed = True
        if 'description' not in data:
            data['description'] = ''
            changed = True
        if changed:
            new_fm = '---\\n' + yaml.safe_dump(data, sort_keys=False).strip() + '\\n---'
            new_lines = [new_fm] + lines[end+1:]
            file_path.write_text('\\n'.join(new_lines), encoding='utf-8')
        title = data.get('title')
        if title:
            if title in titles:
                duplicates.append((file_path, titles[title]))
            else:
                titles[title] = file_path
    else:
        title = next((re.sub(r'^#+\\s*', '', l) for l in lines if l.startswith('#')), file_path.stem)
        new_fm = f"---\\ntitle: {title}\\ndescription: \\n---"
        file_path.write_text(new_fm + '\\n' + text, encoding='utf-8')
        if title in titles:
            duplicates.append((file_path, titles[title]))
        else:
            titles[title] = file_path

def check_links(file_path: Path):
    text = file_path.read_text(encoding='utf-8')
    for match in re.finditer(r'\\[[^\\]]*\\]\\(([^)]+)\\)', text):
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
            f.write('| Source | Target | Error |\\n')
            f.write('| --- | --- | --- |\\n')
            for r in report:
                f.write(f"| {r['source']} | {r['target']} | {r['error']} |\\n")
            for a,b in duplicates:
                f.write(f"| {a} | {b} | duplicate title |\\n")

if __name__ == '__main__':
    main()
'''
    
    # Create scripts directory if it doesn't exist
    scripts_dir = Path('scripts')
    scripts_dir.mkdir(exist_ok=True)
    
    script_path = scripts_dir / 'docs_validation.py'
    script_path.write_text(script_content)
    script_path.chmod(0o755)
    
    print("✓ Created scripts/docs_validation.py")

def add_frontmatter_to_file(file_path: Path, title: str, description: str = ''):
    """Add frontmatter to a markdown file."""
    if not file_path.exists():
        print(f"⚠ File not found: {file_path}")
        return
    
    content = file_path.read_text(encoding='utf-8')
    lines = content.splitlines()
    
    # Check if frontmatter already exists
    if lines and lines[0].strip() == '---':
        # Find closing ---
        try:
            end_idx = lines[1:].index('---') + 1
            # Update existing frontmatter
            fm_lines = lines[1:end_idx]
            try:
                fm_data = yaml.safe_load('\n'.join(fm_lines)) or {}
            except:
                fm_data = {}
            
            fm_data['title'] = title
            if 'description' not in fm_data:
                fm_data['description'] = description
            
            new_fm = yaml.safe_dump(fm_data, sort_keys=False).strip()
            new_content = f"---\n{new_fm}\n---\n" + '\n'.join(lines[end_idx+1:])
        except ValueError:
            # No closing ---, add it
            fm_data = {'title': title, 'description': description}
            new_fm = yaml.safe_dump(fm_data, sort_keys=False).strip()
            new_content = f"---\n{new_fm}\n---\n{content}"
    else:
        # Add new frontmatter
        fm_data = {'title': title, 'description': description}
        new_fm = yaml.safe_dump(fm_data, sort_keys=False).strip()
        new_content = f"---\n{new_fm}\n---\n{content}"
    
    file_path.write_text(new_content, encoding='utf-8')
    print(f"✓ Updated frontmatter: {file_path}")

def update_docusaurus_config():
    """Update docusaurus.config.ts with German locale."""
    config_path = Path('docs/docusaurus.config.ts')
    if not config_path.exists():
        print("⚠ docs/docusaurus.config.ts not found")
        return
    
    content = config_path.read_text(encoding='utf-8')
    
    # Update i18n config
    old_i18n = '''i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },'''
    
    new_i18n = '''i18n: {
  defaultLocale: 'de',
  locales: ['de', 'en'],
},'''
    
    content = content.replace(old_i18n, new_i18n)
    
    # Add locale dropdown to navbar
    navbar_items = '''items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          href: 'https://github.com/EcoSphereNetwork/SmolDesk',
          label: 'GitHub',
          position: 'right',
        },
      ],'''
    
    new_navbar_items = '''items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          href: 'https://github.com/EcoSphereNetwork/SmolDesk',
          label: 'GitHub',
          position: 'right',
        },
        {
          type: 'localeDropdown',
          position: 'right',
        },
      ],'''
    
    content = content.replace(navbar_items, new_navbar_items)
    
    config_path.write_text(content, encoding='utf-8')
    print("✓ Updated docs/docusaurus.config.ts")

def update_sidebars():
    """Update sidebars.ts."""
    sidebar_path = Path('docs/sidebars.ts')
    if not sidebar_path.exists():
        print("⚠ docs/sidebars.ts not found")
        return
    
    content = sidebar_path.read_text(encoding='utf-8')
    
    # Update agents path
    content = content.replace(
        "{ type: 'autogenerated', dirName: 'agents' },",
        "{ type: 'autogenerated', dirName: 'docs/agents' },"
    )
    
    sidebar_path.write_text(content, encoding='utf-8')
    print("✓ Updated docs/sidebars.ts")

def create_english_architecture_doc():
    """Create English version of architecture.md."""
    en_docs_dir = Path('docs/i18n/en/docusaurus-plugin-content-docs/current')
    en_docs_dir.mkdir(parents=True, exist_ok=True)
    
    # Copy architecture.md to English version
    arch_path = Path('docs/docs/architecture.md')
    if arch_path.exists():
        en_arch_path = en_docs_dir / 'architecture.md'
        content = arch_path.read_text(encoding='utf-8')
        en_arch_path.write_text(content, encoding='utf-8')
        print("✓ Created English architecture.md")

def apply_frontmatter_changes():
    """Apply frontmatter changes to all markdown files based on the diff."""
    
    # Mapping of files to their new titles based on the diff
    frontmatter_updates = {
        'docs/AGENTS.md': 'SmolDesk-Mobile Documentation Guide',
        'docs/README.md': 'Website', 
        'docs/archive/old_docs/Build-Anleitung.md': '🚀 SmolDesk Build-Anleitung',
        'docs/archive/old_docs/Dependencies.md': '🚀 **Automatische Installation:**',
        'docs/archive/old_docs/SECURITY.md': 'Sicherheitsarchitektur',
        'docs/archive/old_docs/Smodesk-Mobile-Eingabe.md': 'SmolDesk Mobile Eingabesteuerung',
        'docs/archive/old_docs/Smodesk-Mobile-Security.md': 'OAuth2-Flow',
        'docs/archive/old_docs/Smodesk-Mobile-Testplan.md': 'Testplan für SmolDesk Mobile',
        'docs/archive/old_docs/Smodesk-Mobile-Testprotokoll.md': 'Testprotokoll SmolDesk Mobile',
        'docs/archive/old_docs/Smodesk-Mobile-UX.md': 'UX Leitfaden für SmolDesk Mobile',
        'docs/archive/old_docs/Smodesk-Mobile.md': 'Smodesk-Mobile',
        'docs/archive/old_docs/SmolDesk-Mobile-PlayStore.md': 'SmolDesk Mobile – Play Store Informationen',
        'docs/archive/old_docs/SmolDesk-Mobile-Release.md': 'SmolDesk Mobile Release Checkliste',
        'docs/archive/old_docs/USER_GUIDE.md': 'SmolDesk Benutzerhandbuch',
        'docs/docs-migration-plan.md': 'Documentation Migration Plan',
        'docs/docs/SmolDesk/README.md': 'SmolDesk Dokumentation',
        'docs/docs/SmolDesk/development/Anforderungsanalyse.md': '**1. Funktionale Anforderungen**',
        'docs/docs/SmolDesk/development/Entwickle-Prompt.md': 'Entwickler-Prompt: SmolDesk - Weiterentwicklung des WebRTC-basierten Remote-Desktop-Tools',
        'docs/docs/SmolDesk/development/Entwicklungsplan.md': '**Phase 1: Grundlegende WebRTC-Implementierung (Wochen 1-4)**',
        'docs/docs/SmolDesk/development/Implementation-Plan.md': 'SmolDesk Implementation Plan',
        'docs/docs/SmolDesk/development/Implementation-Status-Update.md': 'SmolDesk Implementation Status Update',
        'docs/docs/SmolDesk/development/Implementation-Status.md': 'SmolDesk Development Plan',
        'docs/docs/SmolDesk/development/Integration&Optimierung.md': 'SmolDesk Integration und Optimierung',
        'docs/docs/SmolDesk/development/Integration-Testing-Plan.md': 'SmolDesk Integration Testing Plan',
        'docs/docs/SmolDesk/development/Leistungsoptimierungsplan.md': 'Leistungsoptimierungsplan für SmolDesk mit modularer Struktur',
        'docs/docs/SmolDesk/development/Optimierungsplan-WebRTC-Bildschirmübertragung.md': 'Optimierungsplan für die WebRTC-Bildschirmübertragung',
        'docs/docs/SmolDesk/development/Technische-Spezifikation.md': '**1. Zielsetzung**',
        'docs/docs/SmolDesk/user/README.md': 'README',
        'docs/docs/agents/README.md': 'Agent Based Development',
        'docs/docs/agents/agent-api-integration.md': 'Agent API Integration',
        'docs/docs/agents/agent-decision-models.md': 'Agent Decision Models',
        'docs/docs/agents/agent-life-cycle.md': 'Agent Life Cycle',
        'docs/docs/agents/agent-safety.md': 'Agent Safety',
        'docs/docs/agents/agent-types.md': 'Agent Types',
        'docs/docs/agents/github-api-access.md': 'GitHub API Access',
        'docs/docs/agents/merge-strategies.md': 'Merge Strategies',
        'docs/docs/agents/pull-request-agent.md': 'Pull Request Agent',
        'docs/docs/api/index.md': 'Internal API',
        'docs/docs/api/ipc-interface.md': 'IPC Interface',
        'docs/docs/api/reference.md': 'API Reference',
        'docs/docs/architecture.md': 'Architecture Overview',
        'docs/docs/components/ClipboardSync.md': 'ClipboardSync',
        'docs/docs/components/ConnectionManager.md': 'ConnectionManager',
        'docs/docs/components/FileTransfer.md': 'FileTransfer',
        'docs/docs/components/RemoteScreen.md': 'RemoteScreen',
        'docs/docs/components/index.md': 'UI Components',
        'docs/docs/components/status.md': 'Komponentenstatus',
        'docs/docs/components/storybook-status.md': 'Storybook Coverage',
        'docs/docs/development/Smodesk-Mobile-Architektur.md': 'SmolDesk Mobile Architektur',
        'docs/docs/development/Smodesk-Mobile-Dateitransfer.md': 'SmolDesk Mobile Dateitransfer',
        'docs/docs/development/Smodesk-Mobile-Signaling.md': 'SmolDesk Mobile Signaling',
        'docs/docs/development/phase-2-report.md': 'Phase 2 Report',
        'docs/docs/development/phase-3-report.md': 'Phase 3 Report',
        'docs/docs/development/phase-4-report.md': 'Phase 4 Report',
        'docs/docs/development/phase-5-overview.md': 'Phase 5 Overview',
        'docs/docs/development/plan.md': 'Development Plan',
        'docs/docs/docusaurus/intro.md': 'Tutorial Intro',
        'docs/docs/docusaurus/tutorial-basics/congratulations.md': 'Congratulations!',
        'docs/docs/docusaurus/tutorial-basics/create-a-blog-post.md': 'Create a Blog Post',
        'docs/docs/docusaurus/tutorial-basics/create-a-document.md': 'Create a Document',
        'docs/docs/docusaurus/tutorial-basics/create-a-page.md': 'Create a Page',
        'docs/docs/docusaurus/tutorial-basics/deploy-your-site.md': 'Deploy your site',
        'docs/docs/docusaurus/tutorial-extras/manage-docs-versions.md': 'Manage Docs Versions',
        'docs/docs/docusaurus/tutorial-extras/translate-your-site.md': 'Translate your site',
        'docs/docs/guides/quickstart.md': 'Quick Start Guide',
        'docs/docs/guides/reorganize.md': 'Repository Reorganization Guide',
        'docs/docs/public/SmolDesk-Mobile-AppStore.md': 'SmolDesk Mobile – App Store Informationen',
        'docs/docs/public/SmolDesk-Mobile-AppStoreRelease.md': 'SmolDesk Mobile App Store Release Checkliste',
        'docs/docs/public/SmolDesk-Mobile-TestFlight.md': 'TestFlight Anleitung',
        'docs/docs/summary.project-insights.md': 'Project Insights',
        'docs/docs/testing/ci-overview.md': 'Continuous Integration Overview',
        'docs/docs/testing/coverage.md': 'Test Coverage',
        'docs/docs/testing/index.md': 'Testing Strategy',
        'docs/docs/testing/phase-4-overview.md': 'Phase 4 Overview',
        'docs/docs/testing/playwright.md': 'Playwright End-to-End Tests',
        'docs/docs/testing/storybook.md': 'Storybook Guide',
        'docs/src/pages/markdown-page.md': 'Markdown page example',
        'docs/testing/strategy.md': 'Testing Strategy'
    }
    
    for file_path, title in frontmatter_updates.items():
        add_frontmatter_to_file(Path(file_path), title)

def run_git_commands():
    """Run git commands to commit and push changes."""
    try:
        # Add all changes
        subprocess.run(['git', 'add', '.'], check=True)
        print("✓ Added all changes to git")
        
        # Commit changes
        commit_message = "docs: Add frontmatter to markdown files and validation script"
        subprocess.run(['git', 'commit', '-m', commit_message], check=True)
        print("✓ Committed changes")
        
        # Push changes
        subprocess.run(['git', 'push'], check=True)
        print("✓ Pushed changes to remote repository")
        
    except subprocess.CalledProcessError as e:
        print(f"✗ Git command failed: {e}")
        return False
    
    return True

def main():
    """Main function to apply all changes."""
    print("📝 Applying documentation frontmatter changes...")
    
    # Check if we're in a git repository
    if not Path('.git').exists():
        print("✗ Not in a git repository. Please run this script from the repository root.")
        return
    
    try:
        # Create new files
        create_markdownlint_config()
        create_docs_validation_script()
        
        # Update existing files
        apply_frontmatter_changes()
        update_docusaurus_config()
        update_sidebars()
        create_english_architecture_doc()
        
        # Run git commands
        print("\n🔄 Committing and pushing changes...")
        if run_git_commands():
            print("\n✅ All changes applied and pushed successfully!")
        else:
            print("\n❌ Failed to push changes. Please check git status manually.")
            
    except Exception as e:
        print(f"✗ Error applying changes: {e}")
        import traceback
        traceback.print_exc()

if __name__ == '__main__':
    main()
