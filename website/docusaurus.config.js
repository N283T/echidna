// @ts-check

import {themes as prismThemes} from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'echidna',
  tagline: 'ChimeraX Bundle Development CLI',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://n283t.github.io',
  baseUrl: '/echidna/',

  organizationName: 'N283T',
  projectName: 'echidna',

  onBrokenLinks: 'throw',

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/N283T/echidna/tree/main/website/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        respectPrefersColorScheme: true,
      },
      navbar: {
        title: 'echidna',
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docsSidebar',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/N283T/echidna',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Documentation',
            items: [
              {
                label: 'Getting Started',
                to: '/docs/getting-started/installation',
              },
              {
                label: 'Commands',
                to: '/docs/commands/init',
              },
              {
                label: 'Configuration',
                to: '/docs/configuration/echidna-toml',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/N283T/echidna',
              },
              {
                label: 'Changelog',
                href: 'https://github.com/N283T/echidna/blob/main/CHANGELOG.md',
              },
              {
                label: 'Contributing',
                to: '/docs/contributing',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} echidna contributors. Built with Docusaurus.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
        additionalLanguages: ['bash', 'toml', 'rust'],
      },
    }),
};

export default config;
