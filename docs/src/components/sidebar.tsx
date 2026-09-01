'use client';

import { cn } from 'cnfast';
import { buttonVariants } from 'fumadocs-ui/components/ui/button';
import {
  Sidebar as SidebarBase,
  type SidebarProps as SidebarBaseProps,
  SidebarProvider,
  SidebarTrigger,
  useSidebar,
} from 'fumadocs-ui/layouts/docs/slots/sidebar';
import Link from 'next/link';
import { GitHubIcon } from '@/components/icons/github';

interface SidebarProps extends SidebarBaseProps {
  mobileOnly?: boolean;
}

export function Sidebar({ mobileOnly, className, ...props }: SidebarProps) {
  const footer = (
    <Link
      href="https://github.com/leegeunhyeok/craby"
      target="_blank"
      className={cn(buttonVariants({ size: 'icon-sm', color: 'ghost' }))}
      aria-label="GitHub"
    >
      <GitHubIcon fill="currentColor" />
    </Link>
  );

  return (
    <SidebarBase {...props} className={cn(className, mobileOnly && 'hidden')} collapsible={false} footer={footer} />
  );
}

function MobileSidebar(props: SidebarBaseProps) {
  return <Sidebar {...props} mobileOnly />;
}

export const sidebarSlots = {
  provider: SidebarProvider,
  root: Sidebar,
  trigger: SidebarTrigger,
  useSidebar,
};

export const mobileSidebarSlots = {
  ...sidebarSlots,
  root: MobileSidebar,
};
