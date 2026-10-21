import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/overview',
        'getting-started/quick-start',
        'getting-started/configuration',
      ],
    },
    {
      type: 'category',
      label: 'Architecture',
      collapsed: false,
      items: [
        'architecture/system-overview',
        'architecture/data-model',
        'architecture/security-and-auth',
        'architecture/clean-room',
      ],
    },
    {
      type: 'category',
      label: 'Media Pipeline',
      collapsed: false,
      items: [
        'media-pipeline/playback-engine',
        'media-pipeline/ffmpeg-execution',
        'media-pipeline/hardware-acceleration',
        'media-pipeline/intro-credit-detection',
        'media-pipeline/mpv-proxy',
      ],
    },
    {
      type: 'category',
      label: 'Clients & Apps',
      collapsed: false,
      items: [
        'clients/overview',
        'clients/web-player',
        'clients/desktop-app',
        'clients/android-app',
      ],
    },
    {
      type: 'category',
      label: 'Deployment & Ops',
      collapsed: true,
      items: [
        'deployment-ops/docker-guide',
        'deployment-ops/packaging-matrix',
        'deployment-ops/backup-and-maintenance',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      collapsed: true,
      items: [
        'api-reference/overview',
        'api-reference/authentication-api',
        'api-reference/media-streaming-api',
        'api-reference/library-management-api',
        'api-reference/server-admin-api',
      ],
    },
  ],
};

export default sidebars;
