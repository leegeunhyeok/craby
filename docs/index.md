---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  image:
    src: /logo.svg
    alt: Craby
  name: Craby
  text: Type-safe Rust for React Native
  tagline: Auto generated, integrated with pure C++ TurboModule
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Introduction
      link: /guide/introduction

features:
  - title: ⚡️ High Performance
    details: Pure C++ TurboModule integration via Rust FFI eliminates platform-specific interop overhead
  - title: 🛡️ Type-Safe Code Generation
    details: Define module specification in TypeScript—auto-generate type-safe bindings
  - title: ✅ Easy Rust + TurboModule Integration
    details: Just implement your own Rust module. Craby handles bridging and platform configuration
---
