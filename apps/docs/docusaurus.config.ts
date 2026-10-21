import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Kinema',
  tagline: 'High-Performance, Self-Hosted Personal Media Server in Rust',
  favicon: 'img/favicon.svg',

  // Set the production url of your site here
  url: 'https://aiyu-ayaan.github.io',
  baseUrl: process.env.DOCS_BASE_URL || '/Kinema/',

  // GitHub pages deployment config.
  organizationName: 'aiyu-ayaan',
  projectName: 'Kinema',
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  markdown: {
    mermaid: true,
  },
  themes: ['@docusaurus/theme-mermaid'],

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/aiyu-ayaan/Kinema/tree/master/apps/docs/',
          showLastUpdateAuthor: false,
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/app-icon.svg',
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Kinema',
      logo: {
        alt: 'Kinema Projector Logo',
        src: 'img/app-icon.svg',
        width: 32,
        height: 32,
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          to: '/docs/architecture/system-overview',
          label: 'Architecture',
          position: 'left',
        },
        {
          to: '/docs/media-pipeline/playback-engine',
          label: 'Media Pipeline',
          position: 'left',
        },
        {
          to: '/docs/clients/overview',
          label: 'Clients & Apps',
          position: 'left',
        },
        {
          to: '/docs/api-reference/overview',
          label: 'API Reference',
          position: 'left',
        },
        {
          href: 'https://github.com/aiyu-ayaan/Kinema',
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
              label: 'Overview & Quickstart',
              to: '/docs/getting-started/overview',
            },
            {
              label: 'System Architecture',
              to: '/docs/architecture/system-overview',
            },
            {
              label: 'Direct-Play Pipeline',
              to: '/docs/media-pipeline/playback-engine',
            },
          ],
        },
        {
          title: 'Ecosystem & Apps',
          items: [
            {
              label: 'Web Player SPA',
              to: '/docs/clients/web-player',
            },
            {
              label: 'Electron Desktop & mpv',
              to: '/docs/clients/desktop-app',
            },
            {
              label: 'Android Jetpack Compose',
              to: '/docs/clients/android-app',
            },
          ],
        },
        {
          title: 'Engineering & Trust',
          items: [
            {
              label: 'Clean-Room Engineering',
              to: '/docs/architecture/clean-room',
            },
            {
              label: 'Security & Threat Model',
              to: '/docs/architecture/security-and-auth',
            },
            {
              label: 'GitHub Repository',
              href: 'https://github.com/aiyu-ayaan/Kinema',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Kinema Project. Clean-room open-source media streaming under MIT License.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['bash', 'json', 'toml', 'rust', 'kotlin', 'sql'],
    },
    mermaid: {
      theme: { light: 'neutral', dark: 'dark' },
      options: {
        fontFamily: 'Inter, system-ui, -apple-system, sans-serif',
      },
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
