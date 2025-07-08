import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'SmolDesk',
  tagline: 'Remote Desktop for Linux',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://ecospherenetwork.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/SmolDesk/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'EcoSphereNetwork',
  projectName: 'SmolDesk',

  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
i18n: {
  defaultLocale: 'de',
  locales: ['de', 'en'],
},

  presets: [
    [
      'classic',
      {
        docs: {
          path: '.',
          routeBasePath: '/',
          sidebarPath: './sidebars.ts',
          editUrl:
            'https://github.com/EcoSphereNetwork/SmolDesk/edit/main/docs/',
          exclude: ['**/node_modules/**'],
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: 'img/docusaurus-social-card.jpg',
    navbar: {
      title: 'SmolDesk',
      logo: {
        alt: 'SmolDesk Logo',
        src: 'img/logo.svg',
      },
      items: [
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
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Project',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/EcoSphereNetwork/SmolDesk',
            },
          ],
        },
      ],
      copyright: `© ${new Date().getFullYear()} SmolDesk Team`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
