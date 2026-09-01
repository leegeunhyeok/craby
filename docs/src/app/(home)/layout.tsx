import { CrabyDocsLayout } from '@/components/docs-layout';
import { HomeNavBar } from '@/components/navbar';
import { source } from '@/lib/source';

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <CrabyDocsLayout
      tree={source.pageTree}
      mobileSidebar
      nav={{ component: <HomeNavBar /> }}
      sidebar={{ collapsible: false, className: '!ps-0' }}
      containerProps={{ className: '!px-2 sm:!px-4 pt-4 md:!px-12 md:pt-[42px] lg:pt-[56px] lg:items-center' }}
      searchToggle={{ enabled: false }}
      themeSwitch={{ enabled: false }}
    >
      {children}
    </CrabyDocsLayout>
  );
}
