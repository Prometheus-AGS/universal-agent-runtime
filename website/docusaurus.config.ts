import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Universal Agent Runtime',
  tagline: 'A tool-first, streaming-native agent runtime for 142+ LLM providers',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  // Production URL for GitHub Pages: https://prometheus-ags.github.io/universal-agent-runtime/
  url: 'https://prometheus-ags.github.io',
  baseUrl: '/universal-agent-runtime/',

  // GitHub Pages deployment config.
  organizationName: 'Prometheus-AGS',
  projectName: 'universal-agent-runtime',
  trailingSlash: false,

  // Ingested docs may reference pages not yet ported; warn rather than fail the build.
  onBrokenLinks: 'warn',

  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl:
            'https://github.com/Prometheus-AGS/universal-agent-runtime/tree/main/website/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/docusaurus-social-card.jpg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Universal Agent Runtime',
      logo: {
        alt: 'UAR Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          href: 'https://github.com/Prometheus-AGS/universal-agent-runtime',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Introduction', to: '/docs/intro'},
            {label: 'Installation', to: '/docs/installation'},
            {label: 'Configuration', to: '/docs/configuration'},
          ],
        },
        {
          title: 'Operations',
          items: [
            {label: 'Backup & Restore', to: '/docs/backup-and-restore'},
            {label: 'Upgrade Guide', to: '/docs/upgrade-guide'},
            {label: 'Troubleshooting', to: '/docs/troubleshooting'},
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/Prometheus-AGS/universal-agent-runtime',
            },
            {
              label: 'Security Policy',
              href: 'https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/SECURITY.md',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Prometheus AGS. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
