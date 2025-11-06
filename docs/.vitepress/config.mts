import { defineConfig, type HeadConfig } from 'vitepress';
import { withMermaid } from 'vitepress-plugin-mermaid';

function getGAScripts(): HeadConfig[] {
  const gaId = process.env.GA_ID;

  if (gaId == null) {
    console.warn('GA_ID is not set');
    return [];
  }

  return [
    [
      'script',
      {
        async: 'true',
        src: `https://www.googletagmanager.com/gtag/js?id=${gaId}`,
      },
    ],
    [
      'script',
      {},
      [
        `window.dataLayer = window.dataLayer || [];`,
        `function gtag(){dataLayer.push(arguments);}`,
        `gtag('js', new Date());`,
        `gtag('config', '${gaId}');`,
      ].join('\n'),
    ],
  ];
}

// https://vitepress.dev/reference/site-config
export default withMermaid(
  defineConfig({
    title: 'Craby',
    description: 'Type-safe Rust for React Native—auto generated, integrated with pure C++ TurboModule',
    head: [
      ['link', { rel: 'icon', href: '/favicon.ico' }],
      ['meta', { property: 'og:image', content: '/banner.png' }],
      ['meta', { name: 'twitter:image', content: '/banner.png' }],
      ...getGAScripts(),
    ],
    themeConfig: {
      // https://vitepress.dev/reference/default-theme-config
      nav: [
        { text: 'Home', link: '/' },
        { text: 'Guide', link: '/guide/introduction' },
      ],
      sidebar: [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/guide/introduction' },
            { text: 'Create a Project', link: '/guide/getting-started' },
            { text: 'Configuration', link: '/guide/configuration' },
            { text: 'Module Definition', link: '/guide/module-definition' },
            { text: 'How to Build', link: '/guide/build' },
            { text: 'CLI Commands', link: '/guide/cli-commands' },
            { text: 'Contributing', link: '/guide/contributing' },
          ],
        },
        {
          text: 'Guides',
          items: [
            { text: 'Types', link: '/guide/types' },
            { text: 'Signals', link: '/guide/signals' },
            { text: 'Errors', link: '/guide/errors' },
            { text: 'Sync vs Async', link: '/guide/sync-vs-async' },
            { text: 'File I/O', link: '/guide/file-io' },
            { text: 'Stateful Modules', link: '/guide/stateful-modules' },
          ],
        },
        {
          items: [
            { text: 'Showcase', link: '/guide/showcase' },
            { text: 'Limitations', link: '/guide/limitations' },
          ],
        },
      ],
      socialLinks: [{ icon: 'github', link: 'https://github.com/leegeunhyeok/craby' }],
      search: {
        provider: 'local',
      },
      footer: {
        message: 'Released under the MIT License.',
        copyright: `Copyright © ${new Date().getFullYear()} Geunhyeok Lee`,
      },
    },
    markdown: {
      theme: {
        dark: 'dark-plus',
        light: 'one-light',
      },
    },
  }),
);
