import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { HomeNavBar } from '@/components/navbar';
import { Sidebar } from '@/components/sidebar';
import { source } from '@/lib/source';

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <DocsLayout
      tree={source.pageTree}
      nav={{ component: <HomeNavBar /> }}
      sidebar={{
        collapsible: false,
        className: '!ps-0',
        component: <Sidebar mobileOnly />,
      }}
      containerProps={{ className: '!px-2 sm:!px-4 pt-4 md:!px-12 md:pt-[42px] lg:pt-[56px] lg:items-center' }}
      searchToggle={{ enabled: false }}
      themeSwitch={{ enabled: false }}
    >
      {children}
    </DocsLayout>
  );
}
