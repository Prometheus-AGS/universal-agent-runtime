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
          routeBasePath: '/docs',
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
          type: 'dropdown',
          label: 'API Reference',
          position: 'left',
          items: [
            { label: 'Rust', href: 'https://prometheus-ags.github.io/universal-agent-runtime/docs/api/rust' },
            { label: 'TypeScript', href: 'https://prometheus-ags.github.io/universal-agent-runtime/docs/api/typescript' },
          ],
        },
        {
          to: '/docs/adr',
          label: 'ADRs',
          position: 'left',
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
            {label: 'Architecture', to: '/docs/architecture/intro'},
            {label: 'Configuration', to: '/docs/configuration/intro'},
            {label: 'Contributing', to: '/docs/contributing/intro'},
          ],
        },
        {
          title: 'SDKs',
          items: [
            {label: 'Rust', to: '/docs/sdk-rust/intro'},
            {label: 'Python', to: '/docs/sdk-python/intro'},
            {label: 'TypeScript', to: '/docs/sdk-typescript/intro'},
          ],
        },
        {
          title: 'Reference',
          items: [
            {label: 'API Reference', to: '/docs/api'},
            {label: 'Architecture Decisions', to: '/docs/adr'},
            {label: 'RAG', to: '/docs/rag/intro'},
            {label: 'A2UI', to: '/docs/a2ui/intro'},
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
