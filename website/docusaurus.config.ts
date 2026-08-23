import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Universal Agent Runtime',
  tagline: 'Governed execution. Typed protocols. One runtime boundary.',
  favicon: 'img/brand/uar-favicon.svg',

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

  onBrokenLinks: 'throw',

  markdown: {
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },

  themes: [
    '@docusaurus/theme-mermaid',
    [
      '@easyops-cn/docusaurus-search-local',
      {
        hashed: 'filename',
        language: ['en'],
        indexDocs: true,
        indexBlog: false,
        indexPages: true,
        docsRouteBasePath: '/docs',
        searchResultLimits: 8,
        searchBarShortcutKeymap: 'mod+k',
      },
    ],
  ],

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
    image: 'img/brand/uar-social-card.svg',
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
    mermaid: {
      theme: {light: 'neutral', dark: 'dark'},
    },
    navbar: {
      title: 'UAR',
      logo: {
        alt: 'Universal Agent Runtime',
        src: 'img/brand/uar-mark-light.svg',
        srcDark: 'img/brand/uar-mark-dark.svg',
        width: 34,
        height: 34,
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          type: 'dropdown',
          label: 'Guides',
          position: 'left',
          items: [
            {label: 'Install UAR', to: '/docs/installation'},
            {label: 'Configure the runtime', to: '/docs/configuration'},
            {label: 'Deploy UAR', to: '/docs/deployment'},
            {label: 'Operate securely', to: '/docs/security'},
          ],
        },
        {
          type: 'dropdown',
          label: 'Reference',
          position: 'left',
          items: [
            {label: 'API overview', to: '/docs/api'},
            {label: 'Rust API', href: 'https://prometheus-ags.github.io/universal-agent-runtime/docs/api/rust'},
            {label: 'TypeScript API', href: 'https://prometheus-ags.github.io/universal-agent-runtime/docs/api/typescript'},
          ],
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
          title: 'Start',
          items: [
            {label: 'Runtime theory', to: '/docs/architecture/intro'},
            {label: 'Install', to: '/docs/installation'},
            {label: 'Configure', to: '/docs/configuration'},
          ],
        },
        {
          title: 'Build',
          items: [
            {label: 'Agents', to: '/docs/intro'},
            {label: 'Skills', to: '/docs/skills'},
            {label: 'Knowledge', to: '/docs/rag/intro'},
          ],
        },
        {
          title: 'Integrate',
          items: [
            {label: 'Rust SDK', to: '/docs/sdk-rust/intro'},
            {label: 'Python SDK', to: '/docs/sdk-python/intro'},
            {label: 'TypeScript SDK', to: '/docs/sdk-typescript/intro'},
          ],
        },
        {
          title: 'Project',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/Prometheus-AGS/universal-agent-runtime',
            },
            {
              label: 'Contributing',
              to: '/docs/contributing/intro',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Prometheus AGS. Universal Agent Runtime is MIT licensed.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
