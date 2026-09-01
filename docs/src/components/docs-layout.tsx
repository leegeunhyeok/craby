'use client';

import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import type { ComponentProps } from 'react';
import { mobileSidebarSlots, sidebarSlots } from './sidebar';

interface CrabyDocsLayoutProps extends ComponentProps<typeof DocsLayout> {
  mobileSidebar?: boolean;
}

export function CrabyDocsLayout({ mobileSidebar, slots, ...props }: CrabyDocsLayoutProps) {
  return (
    <DocsLayout
      {...props}
      slots={{
        ...slots,
        sidebar: mobileSidebar ? mobileSidebarSlots : sidebarSlots,
      }}
    />
  );
}
