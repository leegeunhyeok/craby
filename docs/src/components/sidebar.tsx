import {
  Sidebar as SidebarBase,
  SidebarContent,
  SidebarContentMobile,
  SidebarFooter,
  SidebarHeader,
  SidebarPageTree,
  SidebarTrigger,
  SidebarViewport,
} from 'fumadocs-ui/components/layout/sidebar';
import { ThemeToggle } from 'fumadocs-ui/components/layout/theme-toggle';
import { buttonVariants } from 'fumadocs-ui/components/ui/button';
import { BaseLinkItem } from 'fumadocs-ui/layouts/links';
import { cn } from 'fumadocs-ui/utils/cn';
import { XIcon } from 'lucide-react';
import { GitHubIcon } from '@/components/icons/github';

interface SidebarProps {
  mobileOnly?: boolean;
}

export function Sidebar(props: SidebarProps) {
  const viewport = (
    <SidebarViewport>
      <SidebarPageTree />
    </SidebarViewport>
  );

  const mobile = (
    <SidebarContentMobile>
      <SidebarHeader>
        <div className="flex items-center justify-between gap-1.5 pl-2 text-fd-muted-foreground">
          <ThemeToggle className="p-0" mode="light-dark" />
          <SidebarTrigger
            className={cn(
              buttonVariants({
                color: 'ghost',
                size: 'icon-sm',
                className: 'p-2',
              }),
            )}
          >
            <XIcon />
          </SidebarTrigger>
        </div>
      </SidebarHeader>
      {viewport}
      <SidebarFooter className="border-none">
        <BaseLinkItem
          item={{
            url: 'https://github.com/leegeunhyeok/craby',
            external: true,
          }}
          className={cn(buttonVariants({ size: 'icon', color: 'ghost' }))}
          aria-label="GitHub"
        >
          <GitHubIcon fill="currentColor" />
        </BaseLinkItem>
      </SidebarFooter>
    </SidebarContentMobile>
  );

  const content = <SidebarContent className="bg-fd-background">{viewport}</SidebarContent>;

  return <SidebarBase Mobile={mobile} Content={props.mobileOnly ? null : content} />;
}
