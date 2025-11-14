'use client';

import { LargeSearchToggle, SearchToggle } from 'fumadocs-ui/components/layout/search-toggle';
import { ThemeToggle } from 'fumadocs-ui/components/layout/theme-toggle';
import { buttonVariants } from 'fumadocs-ui/components/ui/button';
import { BaseLinkItem } from 'fumadocs-ui/layouts/links';
import { useSidebar } from 'fumadocs-ui/provider';
import { cn } from 'fumadocs-ui/utils/cn';
import { MenuIcon } from 'lucide-react';
import Link from 'next/link';
import { useIsScrolled } from '@/hooks/use-is-scrolled';
import { GitHubIcon } from './icons/github';
import { Title } from './title';

const iconButtonClass = buttonVariants({ variant: 'ghost', size: 'icon', className: 'cursor-pointer' });

const HOME_LINKS = [
  {
    label: 'Documentation',
    url: '/docs/get-started/introduction',
  },
];

export function HomeNavBar() {
  return <Navbar mode="home" links={HOME_LINKS} />;
}

export function DocsNavBar() {
  return <Navbar mode="docs" />;
}

interface NavbarProps {
  mode: 'home' | 'docs';
  links?: {
    label: string;
    url: string;
  }[];
}

function Navbar({ mode, links }: NavbarProps) {
  const { open, setOpen } = useSidebar();
  const isScrolled = useIsScrolled();

  const right = () => {
    return (
      <div className="flex flex-row items-center justify-center gap-1.5 pl-2">
        <Title />
        <div className="flex flex-row items-center justify-center gap-1.5 max-md:hidden">
          {links?.map((link) => (
            <BaseLinkItem
              key={link.label}
              item={{ url: link.url }}
              className="text-fd-muted-foreground text-sm hover:text-fd-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fd-ring"
            >
              {link.label}
            </BaseLinkItem>
          ))}
        </div>
      </div>
    );
  };

  const left = () => {
    return (
      <div className="flex flex-row items-center justify-center gap-1.5">
        <Link
          href="https://github.com/leegeunhyeok/craby"
          target="_blank"
          className={`max-md:hidden ${iconButtonClass}`}
        >
          <GitHubIcon fill="currentColor" />
        </Link>
        {/* Desktop */}
        <LargeSearchToggle className="inline-flex hidden w-[200px] cursor-pointer items-center gap-2 rounded-full border bg-fd-secondary/50 p-1.5 ps-2 text-fd-muted-foreground text-sm transition-colors hover:bg-fd-accent hover:text-fd-accent-foreground md:flex dark:bg-[#2e2e2e]" />
        <ThemeToggle className="hidden cursor-pointer md:flex" mode="light-dark" />
        {/* Mobile */}
        <SearchToggle className="cursor-pointer md:hidden" />
        <div className="flex flex-row items-center justify-center md:hidden">
          <button type="button" className={iconButtonClass} onClick={() => setOpen(!open)}>
            <MenuIcon color="currentColor" />
          </button>
        </div>
      </div>
    );
  };

  return (
    <nav
      className={cn(
        'fixed top-0 z-10 flex h-[56px] w-full bg-fd-background transition-[border] duration-100 md:z-1000',
        (mode === 'docs' || isScrolled) && 'border-fd-border border-b',
      )}
    >
      <div
        className={cn(
          'mx-auto flex w-full flex-row items-center justify-between px-4',
          mode === 'home' && 'max-w-[1200px]',
        )}
      >
        {right()}
        {left()}
      </div>
    </nav>
  );
}
