import { buttonVariants } from 'fumadocs-ui/components/ui/button';
import { cn } from 'fumadocs-ui/utils/cn';
import Link from 'next/link';
import { type PropsWithChildren, Suspense } from 'react';
import { CodePreview, CodePreviewFallback } from '@/components/code-preview';
import { Feature } from '@/components/feature';

export default function HomePage() {
  return (
    <div className="flex max-w-[1200px] flex-1 flex-col gap-4 p-4 xs:p-8 pt-16 text-center lg:pt-20">
      <section className="flex flex-row items-center justify-center max-[1100px]:flex-col max-[1100px]:gap-15">
        <div className="flex max-w-[600px] flex-col items-start justify-center whitespace-pre-wrap text-left max-[1100px]:items-center">
          <p className="w-fit bg-[linear-gradient(120deg,#82d7f7_10%,#387ca0)] bg-clip-text font-bold text-4xl text-transparent leading-12 tracking-tight antialiased max-[1100px]:text-center sm:text-5xl sm:leading-15 md:text-6xl md:leading-18">
            Craby
          </p>
          <p className="leading:10 sm:leading:12 font-bold text-4xl text-fd-foreground-secondary tracking-tight antialiased max-[1100px]:text-center sm:text-5xl md:text-6xl md:leading-15">
            Type-safe Rust for React Native
          </p>
          <p className="mt-2 text-fd-muted-foreground text-lg max-[1100px]:text-center md:mt-4 md:text-2xl">
            Auto generated, integrated with pure C++ TurboModule
          </p>
          <div className="mt-6 flex flex-row gap-4">
            <Link
              href="/docs/get-started/create-project"
              type="button"
              className={cn(buttonVariants({ variant: 'primary' }), 'cursor-pointer rounded-full px-4 text-md')}
            >
              Get Started
            </Link>
            <Link
              href="/docs/get-started/introduction"
              type="button"
              className={cn(buttonVariants({ variant: 'outline' }), 'cursor-pointer rounded-full px-4 text-md')}
            >
              Introduction
            </Link>
          </div>
        </div>
        <div className="flex w-full max-w-[600px] pl-8 drop-shadow-[0_0_16px_rgba(130,215,247,0.8)] xs:drop-shadow-[0_0_25px_rgba(130,215,247,0.8)] max-[1100px]:pl-0">
          <Suspense fallback={<CodePreviewFallback />}>
            <CodePreview />
          </Suspense>
        </div>
      </section>
      <section className="mt-16 max-[1100px]:mt-12">
        <div className="mx-auto grid grid-cols-3 gap-4 max-[1100px]:max-w-[600px] max-[1100px]:grid-cols-1 max-[1100px]:gap-8">
          <Feature title="High Performance" emoji="⚡️">
            Pure C++ TurboModule integration via Rust FFI eliminates platform-specific interop overhead
          </Feature>
          <Feature title="Type-Safe Code Generation" emoji="🛡️">
            Define module specification in TypeScript—auto-generate type-safe bindings
          </Feature>
          <Feature title="Easy Rust + TurboModule Integration" emoji="✅">
            Just implement your own Rust module. Craby handles bridging and platform configuration
          </Feature>
        </div>
      </section>
      <section className="mt-20 mb-16 max-[1100px]:mt-4 max-[1100px]:mb-12">
        <div className="mx-auto max-w-[800px] rounded-2xl border border-fd-border bg-fd-card p-4 max-[1100px]:max-w-[600px] sm:p-8 md:p-12">
          <h2 className="font-bold text-fd-foreground text-xl tracking-tight sm:text-3xl md:text-4xl">
            Rust's Power, Native Integration
          </h2>
          <p className="mt-4 text-center text-base text-gray-500 leading-relaxed sm:mt-6 sm:text-lg md:text-xl">
            <b className="font-semibold text-fd-primary">Craby</b> is a type-safe Rust integration framework for React
            Native developers. Bringing high-performance native modules with automatic code generation and seamless C++
            TurboModule integration—making Rust accessible to JavaScript developers.
          </p>
        </div>
      </section>
      <footer className="mt-auto border-fd-border border-t bg-fd-background pt-8">
        <div className="mx-auto max-[1100px]:max-w-[600px]">
          <div className="grid grid-cols-1 gap-8 md:grid-cols-3">
            <div className="flex flex-col gap-2">
              <p className="font-semibold text-fd-foreground text-sm">Documentation</p>
              <FooterLink href="/docs/get-started/introduction">Introduction</FooterLink>
              <FooterLink href="/docs/get-started/create-project">Get Started</FooterLink>
            </div>
            <div className="flex flex-col gap-2">
              <p className="font-semibold text-fd-foreground text-sm">Community</p>
              <FooterLink href="https://github.com/leegeunhyeok/craby" target="_blank" rel="noopener noreferrer">
                GitHub
              </FooterLink>
              <FooterLink
                href="https://github.com/leegeunhyeok/craby/discussions"
                target="_blank"
                rel="noopener noreferrer"
              >
                Discussions
              </FooterLink>
            </div>
            <div className="flex flex-col gap-2">
              <p className="font-semibold text-fd-foreground text-sm">Resources</p>
              <FooterLink
                href="https://github.com/leegeunhyeok/craby/blob/main/LICENSE"
                target="_blank"
                rel="noopener noreferrer"
              >
                License
              </FooterLink>
              <FooterLink
                href="https://github.com/leegeunhyeok/craby/releases"
                target="_blank"
                rel="noopener noreferrer"
              >
                Releases
              </FooterLink>
            </div>
          </div>
          <div className="mt-8 pt-4 text-center">
            <p className="text-fd-muted-foreground text-sm">
              Copyright © {new Date().getFullYear()}{' '}
              <Link
                href="https://github.com/leegeunhyeok"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-fd-accent-foreground"
              >
                Geunhyeok LEE
              </Link>
              . All rights reserved.
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
}

function FooterLink({
  href,
  children,
  target,
  rel,
}: PropsWithChildren<{ href: string; target?: string; rel?: string }>) {
  return (
    <Link
      className="text-fd-muted-foreground text-sm hover:text-fd-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fd-ring"
      href={href}
      target={target}
      rel={rel}
    >
      {children}
    </Link>
  );
}
