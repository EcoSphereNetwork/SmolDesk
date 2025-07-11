import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'SmolDesk',
  tagline: 'Remote Desktop for Linux',
  favicon: 'img/32x32.png',
  
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
  themes: ['@docusaurus/theme-live-codeblock'],

  themeConfig: {
    colorMode: {
      defaultMode: 'light',
    },
    // Replace with your project's social card
    image: 'img/docusaurus-social-card.jpg',
    navbar: {
      title: 'SmolDesk',
      logo: {
        alt: 'SmolDesk Logo',
        src: 'img/smoldesk-logo.png',
        href: '/',
      },
      items: [
        {to: '/', label: 'Home', position: 'left'},
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Dokumentation',
        },
        {to: '/api/index', label: 'API', position: 'left'},
        {to: '/release/changelog', label: 'Releases', position: 'left'},
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
          title: 'Links',
          items: [
            {label: 'Docs', to: '/'},
            {label: 'API', to: '/api/index'},
            {label: 'GitHub', href: 'https://github.com/EcoSphereNetwork/SmolDesk'},
          ],
        },
        {
          title: 'Infos',
          items: [
            {label: 'Lizenz', href: 'https://github.com/EcoSphereNetwork/SmolDesk/blob/main/LICENSE'},
            {label: 'Datenschutz', to: '/docs/public/privacy-policy.html'},
          ],
        },
        {
          title: 'Mitmachen',
          items: [
            {label: 'GitHub Issues', href: 'https://github.com/EcoSphereNetwork/SmolDesk/issues'},
            {label: 'Discord', href: 'https://discord.gg/smoldesk'},
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
