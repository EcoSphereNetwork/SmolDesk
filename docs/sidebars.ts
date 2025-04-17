import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  wikiSidebar: [
    {
      type: 'category',
      label: 'Guides',
      collapsed: true,
      items: [
        'guides/quickstart',
        'guides/reorganize',
      ],
    },
    {
      type: 'category',
      label: 'Development',
      collapsed: true,
      items: [
        'development/guide',
        'development/roadmap',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      collapsed: true,
      items: ['api/reference'],
    },
  ],
  tutorialSidebar: [
    'docusaurus/intro',
    {
      type: 'category',
      label: 'Tutorial Basics',
      collapsed: false,
      items: [
        'docusaurus/tutorial-basics/create-a-document',
        'docusaurus/tutorial-basics/create-a-blog-post',
        'docusaurus/tutorial-basics/create-a-page',
        'docusaurus/tutorial-basics/deploy-your-site',
        'docusaurus/tutorial-basics/congratulations',
        'docusaurus/tutorial-basics/markdown-features',
      ],
    },
    {
      type: 'category',
      label: 'Tutorial Extras',
      collapsed: false,
      items: [
        'docusaurus/tutorial-extras/manage-docs-versions',
        'docusaurus/tutorial-extras/translate-your-site',
      ],
    },
  ],
};

export default sidebars;
