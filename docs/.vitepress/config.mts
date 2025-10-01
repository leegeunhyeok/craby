import { defineConfig } from 'vitepress';
import { withMermaid } from 'vitepress-plugin-mermaid';

// https://vitepress.dev/reference/site-config
export default withMermaid(
  defineConfig({
    title: 'Craby',
    description: 'Type-safe Rust for React Native—auto generated, integrated with pure C++ TurboModule',
    head: [['link', { rel: 'icon', href: '/favicon.ico' }]],
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
            { text: 'Setup Craby Project', link: '/guide/getting-started' },
            { text: 'Module Definition', link: '/guide/module-definition' },
            { text: 'How to build', link: '/guide/build' },
            { text: 'CLI Commands', link: '/guide/cli-commands' },
          ],
        },
        {
          text: 'Guides',
          items: [
            { text: 'Types', link: '/guide/types' },
            { text: 'Signals', link: '/guide/signals' },
            { text: 'Errors', link: '/guide/errors' },
            { text: 'Sync vs Async', link: '/guide/sync-vs-async' },
          ],
        },
      ],
      socialLinks: [{ icon: 'github', link: 'https://github.com/leegeunhyeok/craby' }],
      search: {
        provider: 'local',
      },
      footer: {
        message: 'Released under the MIT License.',
        copyright: 'Copyright © 2025 Geunhyeok Lee',
      },
    },
  }),
);
