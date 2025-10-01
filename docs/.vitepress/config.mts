import { defineConfig } from 'vitepress';
import { withMermaid } from 'vitepress-plugin-mermaid';

// https://vitepress.dev/reference/site-config
export default withMermaid(defineConfig({
  title: 'Craby',
  description: 'Type-safe Rust for React Native—auto generated, integrated with pure C++ TurboModule',
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/introduction' },
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Introduction', link: '/guide/introduction' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Module Definition', link: '/guide/module-definition' },
          { text: 'Code Generation', link: '/guide/codegen' },
          { text: 'Building', link: '/guide/building' },
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
}));
