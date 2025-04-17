import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/__docusaurus/debug',
    component: ComponentCreator('/__docusaurus/debug', '5ff'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/config',
    component: ComponentCreator('/__docusaurus/debug/config', '5ba'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/content',
    component: ComponentCreator('/__docusaurus/debug/content', 'a2b'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/globalData',
    component: ComponentCreator('/__docusaurus/debug/globalData', 'c3c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/metadata',
    component: ComponentCreator('/__docusaurus/debug/metadata', '156'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/registry',
    component: ComponentCreator('/__docusaurus/debug/registry', '88c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/routes',
    component: ComponentCreator('/__docusaurus/debug/routes', '000'),
    exact: true
  },
  {
    path: '/blog',
    component: ComponentCreator('/blog', 'b2f'),
    exact: true
  },
  {
    path: '/blog/archive',
    component: ComponentCreator('/blog/archive', '182'),
    exact: true
  },
  {
    path: '/blog/authors',
    component: ComponentCreator('/blog/authors', '0b7'),
    exact: true
  },
  {
    path: '/blog/authors/all-sebastien-lorber-articles',
    component: ComponentCreator('/blog/authors/all-sebastien-lorber-articles', '4a1'),
    exact: true
  },
  {
    path: '/blog/authors/yangshun',
    component: ComponentCreator('/blog/authors/yangshun', 'a68'),
    exact: true
  },
  {
    path: '/blog/first-blog-post',
    component: ComponentCreator('/blog/first-blog-post', '89a'),
    exact: true
  },
  {
    path: '/blog/long-blog-post',
    component: ComponentCreator('/blog/long-blog-post', '9ad'),
    exact: true
  },
  {
    path: '/blog/mdx-blog-post',
    component: ComponentCreator('/blog/mdx-blog-post', 'e9f'),
    exact: true
  },
  {
    path: '/blog/tags',
    component: ComponentCreator('/blog/tags', '287'),
    exact: true
  },
  {
    path: '/blog/tags/docusaurus',
    component: ComponentCreator('/blog/tags/docusaurus', '704'),
    exact: true
  },
  {
    path: '/blog/tags/facebook',
    component: ComponentCreator('/blog/tags/facebook', '858'),
    exact: true
  },
  {
    path: '/blog/tags/hello',
    component: ComponentCreator('/blog/tags/hello', '299'),
    exact: true
  },
  {
    path: '/blog/tags/hola',
    component: ComponentCreator('/blog/tags/hola', '00d'),
    exact: true
  },
  {
    path: '/blog/welcome',
    component: ComponentCreator('/blog/welcome', 'd2b'),
    exact: true
  },
  {
    path: '/markdown-page',
    component: ComponentCreator('/markdown-page', '3d7'),
    exact: true
  },
  {
    path: '/docs',
    component: ComponentCreator('/docs', '9a1'),
    routes: [
      {
        path: '/docs',
        component: ComponentCreator('/docs', '94f'),
        routes: [
          {
            path: '/docs',
            component: ComponentCreator('/docs', '6bb'),
            routes: [
              {
                path: '/docs/api/reference',
                component: ComponentCreator('/docs/api/reference', '359'),
                exact: true,
                sidebar: "wikiSidebar"
              },
              {
                path: '/docs/development/guide',
                component: ComponentCreator('/docs/development/guide', '280'),
                exact: true,
                sidebar: "wikiSidebar"
              },
              {
                path: '/docs/development/roadmap',
                component: ComponentCreator('/docs/development/roadmap', '49a'),
                exact: true,
                sidebar: "wikiSidebar"
              },
              {
                path: '/docs/docusaurus/intro',
                component: ComponentCreator('/docs/docusaurus/intro', '6c3'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-basics/congratulations',
                component: ComponentCreator('/docs/docusaurus/tutorial-basics/congratulations', '0d3'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-basics/create-a-blog-post',
                component: ComponentCreator('/docs/docusaurus/tutorial-basics/create-a-blog-post', '642'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-basics/create-a-document',
                component: ComponentCreator('/docs/docusaurus/tutorial-basics/create-a-document', '2db'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-basics/create-a-page',
                component: ComponentCreator('/docs/docusaurus/tutorial-basics/create-a-page', 'b41'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-basics/deploy-your-site',
                component: ComponentCreator('/docs/docusaurus/tutorial-basics/deploy-your-site', '78b'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-basics/markdown-features',
                component: ComponentCreator('/docs/docusaurus/tutorial-basics/markdown-features', 'e3c'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-extras/manage-docs-versions',
                component: ComponentCreator('/docs/docusaurus/tutorial-extras/manage-docs-versions', 'c24'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/docusaurus/tutorial-extras/translate-your-site',
                component: ComponentCreator('/docs/docusaurus/tutorial-extras/translate-your-site', '512'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/guides/quickstart',
                component: ComponentCreator('/docs/guides/quickstart', 'd16'),
                exact: true,
                sidebar: "wikiSidebar"
              },
              {
                path: '/docs/guides/reorganize',
                component: ComponentCreator('/docs/guides/reorganize', '426'),
                exact: true,
                sidebar: "wikiSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '/',
    component: ComponentCreator('/', 'e5f'),
    exact: true
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
