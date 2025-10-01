---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  image:
    src: /logo.svg
    alt: Craby
  name: "Craby"
  text: "Type-safe Rust for React Native"
  tagline: Auto-generated, integrated with pure C++ TurboModule
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Introduction
      link: /guide/introduction

features:
  - title: ⚡️ High Performance
    details: Pure C++ integration with zero-cost FFI eliminates platform-specific interop overhead
  - title: 🛡️ Type-Safe Code Generation
    details: Define APIs in TypeScript—auto-generate type-safe Rust traits and C++ bridges
  - title: ✅ Easy Rust + TurboModule Integration
    details: Just implement Rust traits. Craby handles bridging and platform configuration
---
