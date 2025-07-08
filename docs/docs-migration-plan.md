# Documentation Migration Plan

This document outlines the planned migration of existing SmolDesk documentation to a simplified
folder hierarchy under `docs/`. Each source file is categorized and mapped to a new target
location. Obsolete template files are marked for archival.

## Planned Target Structure

- `docs/architecture/`
- `docs/components/`
- `docs/development/`
- `docs/usage/`
- `docs/setup/`
- `docs/testing/`
- `docs/release/`
- `docs/agents/`
- `docs/api/`
- `docs/archive/` (template or outdated files)

## File Mapping

| Source Path | Category | Planned Target | Notes |
|-------------|----------|----------------|-------|
| docs/docs/summary.project-insights.md | Feature | docs/architecture/summary-project-insights.md |
| docs/docs/architecture.md | Architecture | docs/architecture/overview.md |
| docs/docs/guides/reorganize.md | Archivwürdig | docs/archive/reorganize.md |
| docs/docs/guides/quickstart.md | Archivwürdig | docs/archive/quickstart.md |
| docs/docs/components/ClipboardSync.md | Feature | docs/components/ClipboardSync.md |
| docs/docs/components/FileTransfer.md | Feature | docs/components/FileTransfer.md |
| docs/docs/components/ConnectionManager.md | Feature | docs/components/ConnectionManager.md |
| docs/docs/components/index.md | Feature | docs/components/index.md |
| docs/docs/components/storybook-status.md | Feature | docs/components/storybook-status.md |
| docs/docs/components/status.md | Feature | docs/components/status.md |
| docs/docs/components/RemoteScreen.md | Feature | docs/components/RemoteScreen.md |
| docs/docs/SmolDesk/README.md | Overview | docs/overview.md |
| docs/docs/development/Smodesk-Mobile-Dateitransfer.md | Feature | docs/development/mobile-dateitransfer.md |
| docs/docs/development/Smodesk-Mobile-Architektur.md | Architecture | docs/development/mobile-architektur.md |
| docs/docs/development/phase-5-overview.md | Development | docs/development/phase-5-overview.md |
| docs/docs/development/phase-4-report.md | Development | docs/development/phase-4-report.md |
| docs/docs/development/phase-3-report.md | Development | docs/development/phase-3-report.md |
| docs/docs/development/Smodesk-Mobile-Security.md | Development | docs/development/mobile-security.md |
| docs/docs/development/Smodesk-Mobile.md | Development | docs/development/mobile-overview.md |
| docs/docs/development/Smodesk-Mobile-Testprotokoll.md | Test | docs/testing/mobile-testprotokoll.md |
| docs/docs/development/plan.md | Development | docs/development/plan.md |
| docs/docs/development/Smodesk-Mobile-UX.md | Feature | docs/usage/mobile-ux.md |
| docs/docs/development/phase-2-report.md | Development | docs/development/phase-2-report.md |
| docs/docs/development/Smodesk-Mobile-Eingabe.md | Feature | docs/usage/mobile-eingabe.md |
| docs/docs/development/Smodesk-Mobile-Testplan.md | Test | docs/testing/mobile-testplan.md |
| docs/docs/development/Smodesk-Mobile-Signaling.md | Development | docs/development/mobile-signaling.md |
| docs/docs/agents/github-api-access.md | Agents | docs/agents/github-api-access.md |
| docs/docs/agents/agent-decision-models.md | Agents | docs/agents/agent-decision-models.md |
| docs/docs/agents/README.md | Agents | docs/agents/README.md |
| docs/docs/agents/merge-strategies.md | Agents | docs/agents/merge-strategies.md |
| docs/docs/agents/agent-types.md | Agents | docs/agents/agent-types.md |
| docs/docs/agents/agent-api-integration.md | Agents | docs/agents/agent-api-integration.md |
| docs/docs/agents/agent-life-cycle.md | Agents | docs/agents/agent-life-cycle.md |
| docs/docs/agents/agent-safety.md | Agents | docs/agents/agent-safety.md |
| docs/docs/agents/pull-request-agent.md | Agents | docs/agents/pull-request-agent.md |
| docs/docs/api/reference.md | API | docs/api/reference.md |
| docs/docs/api/index.md | API | docs/api/index.md |
| docs/docs/api/ipc-interface.md | API | docs/api/ipc-interface.md |
| docs/docs/testing/storybook.md | Test | docs/testing/storybook.md |
| docs/docs/testing/phase-4-overview.md | Test | docs/testing/phase-4-overview.md |
| docs/docs/testing/playwright.md | Test | docs/testing/playwright.md |
| docs/docs/testing/index.md | Test | docs/testing/index.md |
| docs/docs/testing/coverage.md | Test | docs/testing/coverage.md |
| docs/docs/testing/ci-overview.md | Test | docs/testing/ci-overview.md |
| docs/docs/public/privacy-policy-ios.html | Release | docs/release/privacy-policy-ios.html |
| docs/docs/public/SmolDesk-Mobile-Release.md | Release | docs/release/mobile-release-checklist.md |
| docs/docs/public/SmolDesk-Mobile-StoreText.json | Release | docs/release/mobile-store-text.json |
| docs/docs/public/privacy-policy.html | Release | docs/release/privacy-policy.html |
| docs/docs/public/SmolDesk-Mobile-AppStore.md | Release | docs/release/mobile-appstore.md |
| docs/docs/public/SmolDesk-Mobile-AppStoreRelease.md | Release | docs/release/mobile-appstore-release.md |
| docs/docs/public/SmolDesk-Mobile-TestFlight.md | Release | docs/release/mobile-testflight.md |
| docs/docs/public/SmolDesk-Mobile-PlayStore.md | Release | docs/release/mobile-playstore.md |
| docs/docs/docusaurus/intro.md | Archivwürdig | docs/archive/docusaurus-intro.md |
| docs/docs/SmolDesk/development/Implementation-Plan.md | Development | docs/development/implementation-plan.md |
| docs/docs/SmolDesk/development/Technische-Spezifikation.md | Architecture | docs/architecture/technische-spezifikation.md |
| docs/docs/SmolDesk/development/Implementation-Status.md | Development | docs/development/implementation-status.md |
| docs/docs/SmolDesk/development/Implementation-Status-Update.md | Development | docs/development/implementation-status-update.md |
| docs/docs/SmolDesk/development/Anforderungsanalyse.md | Development | docs/development/anforderungsanalyse.md |
| docs/docs/SmolDesk/development/Integration-Testing-Plan.md | Test | docs/testing/integration-testing-plan.md |
| docs/docs/SmolDesk/development/Leistungsoptimierungsplan.md | Development | docs/development/leistungsoptimierungsplan.md |
| docs/docs/SmolDesk/development/Entwicklungsplan.md | Development | docs/development/entwicklungsplan.md |
| docs/docs/SmolDesk/development/Integration&Optimierung.md | Development | docs/development/integration-und-optimierung.md |
| docs/docs/SmolDesk/development/Optimierungsplan-WebRTC-Bildschirmübertragung.md | Development | docs/development/optimierungsplan-webrtc-bildschirmuebertragung.md |
| docs/docs/SmolDesk/development/Entwickle-Prompt.md | Development | docs/development/entwickle-prompt.md |
| docs/docs/SmolDesk/user/Dependencies.md | Setup | docs/setup/dependencies.md |
| docs/docs/SmolDesk/user/README.md | Archivwürdig | docs/archive/user-readme.md |
| docs/docs/SmolDesk/user/Build-Anleitung.md | Setup | docs/setup/build-anleitung.md |
| docs/docs/SmolDesk/user/USER_GUIDE.md | Usage | docs/usage/user-guide.md |
| docs/docs/SmolDesk/user/SECURITY.md | Security | docs/architecture/security.md |
| docs/docs/SmolDesk/static/img/logo.png | Asset | docs/assets/logo.png |
| docs/docs/docusaurus/tutorial-extras/translate-your-site.md | Archivwürdig | docs/archive/translate-your-site.md |
| docs/docs/docusaurus/tutorial-extras/manage-docs-versions.md | Archivwürdig | docs/archive/manage-docs-versions.md |
| docs/docs/docusaurus/tutorial-extras/_category_.json | Archivwürdig | docs/archive/tutorial-extras-category.json |
| docs/docs/docusaurus/tutorial-basics/congratulations.md | Archivwürdig | docs/archive/congratulations.md |
| docs/docs/docusaurus/tutorial-basics/markdown-features.mdx | Archivwürdig | docs/archive/markdown-features.mdx |
| docs/docs/docusaurus/tutorial-basics/_category_.json | Archivwürdig | docs/archive/tutorial-basics-category.json |
| docs/docs/docusaurus/tutorial-basics/create-a-blog-post.md | Archivwürdig | docs/archive/create-a-blog-post.md |
| docs/docs/docusaurus/tutorial-basics/deploy-your-site.md | Archivwürdig | docs/archive/deploy-your-site.md |
| docs/docs/docusaurus/tutorial-basics/create-a-page.md | Archivwürdig | docs/archive/create-a-page.md |
| docs/docs/docusaurus/tutorial-basics/create-a-document.md | Archivwürdig | docs/archive/create-a-document.md |
| docs/docs/docusaurus/tutorial-extras/img/localeDropdown.png | Archivwürdig | docs/archive/localeDropdown.png |
| docs/docs/docusaurus/tutorial-extras/img/docsVersionDropdown.png | Archivwürdig | docs/archive/docsVersionDropdown.png |

## Migration Steps

1. **Consolidate UX guides**
   - Move `Smodesk-Mobile-UX.md`, `Smodesk-Mobile-Eingabe.md` and related user docs to `docs/usage/`.
   - Update internal links and navigation.
2. **Setup documentation**
   - Migrate `Dependencies.md` and `Build-Anleitung.md` to `docs/setup/`.
   - Merge overlapping instructions from `README.md` if needed.
3. **Development and architecture docs**
   - Relocate all phase reports, plans and architecture descriptions to `docs/development/` and `docs/architecture/`.
   - Ensure cross-links between phases remain intact.
4. **Component and API references**
   - Place UI component docs under `docs/components/`.
   - Move existing API reference files to `docs/api/`.
5. **Testing documents**
   - Gather test plans, coverage notes and CI guides under `docs/testing/`.
6. **Release notes and store texts**
   - Consolidate public release checklists, privacy policies and store descriptions under `docs/release/`.
7. **Archive template files**
   - Move Docusaurus tutorial content and unused user README to `docs/archive/` for reference.


## Open Questions

- `docs/docs/SmolDesk/README.md` partially duplicates the project `README.md`. Review content to merge or replace.
- Docusaurus templates under `docs/docs/docusaurus/` are untouched upstream; decide if they can be removed entirely after migration.
- Some phase reports reference images not present in the repository; verify if assets are missing or obsolete.

