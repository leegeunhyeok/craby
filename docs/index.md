---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
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
  - title: ⚡️ Faster Than Standard TurboModules
    details: Pure C++ integration with zero-cost FFI bridges Rust and C++ at compile-time using templates, eliminating platform-specific interop overhead (ObjCTurboModule, JavaTurboModule)
  - title: 🛡️ Type-Safe Code Generation
    details: Define your API once in TypeScript—Craby automatically generates type-safe Rust traits, C++ bridges, and FFI layers with compile-time validation across the entire stack
  - title: 🦀 Easy Rust + TurboModule Integration
    details: Just implement the generated Rust trait with your business logic. Craby handles all the complex bridging, building, and platform configuration automatically
---

