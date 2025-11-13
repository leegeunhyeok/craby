'use client';

import { LargeSearchToggle, SearchToggle } from 'fumadocs-ui/components/layout/search-toggle';
import { buttonVariants } from 'fumadocs-ui/components/ui/button';
import { useSidebar } from 'fumadocs-ui/provider';
import { MenuIcon } from 'lucide-react';
import { Title } from './title';
import Link from 'next/link';
import { cn } from 'fumadocs-ui/utils/cn';
import { useIsScrolled } from '@/hooks/use-is-scrolled';
import { BaseLinkItem } from 'fumadocs-ui/layouts/links';

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
      <div className="flex flex-row items-center justify-center gap-1.5">
        <Title />
        {links?.map((link) => (
          <BaseLinkItem
            key={link.label}
            item={{ url: link.url }}
            className="text-sm text-fd-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-fd-ring hover:text-fd-accent-foreground"
          >
            {link.label}
          </BaseLinkItem>
        ))}
      </div>
    );
  };

  return (
    <nav
      className={cn(
        'bg-fd-background w-full h-[56px] fixed top-0 z-10 md:z-1000 px-4 flex flex-row justify-between items-center',
        (mode === 'docs' || isScrolled) && 'border-b border-fd-border',
      )}
    >
      {right()}
      <div className="flex flex-row items-center justify-center gap-1.5">
        <Link
          href="https://github.com/leegeunhyeok/craby"
          target="_blank"
          className={`max-md:hidden ${iconButtonClass}`}
        >
          <GitHubIcon />
        </Link>
        <LargeSearchToggle className="w-[200px] inline-flex items-center gap-2 rounded-full border bg-fd-secondary/50 dark:bg-[#2e2e2e] p-1.5 ps-2 text-sm text-fd-muted-foreground transition-colors hover:bg-fd-accent hover:text-fd-accent-foreground hidden md:flex cursor-pointer" />
        <SearchToggle className="md:hidden cursor-pointer" />
        <div className="flex flex-row items-center justify-center md:hidden">
          <button type="button" className={iconButtonClass} onClick={() => setOpen(!open)}>
            <MenuIcon color="currentColor" />
          </button>
        </div>
      </div>
    </nav>
  );
}

// https://simpleicons.org/?q=github
function GitHubIcon() {
  return (
    <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" style={{ fill: 'currentColor' }}>
      <title>GitHub</title>
      <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
    </svg>
  );
}
