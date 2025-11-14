import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { DocsNavBar } from '@/components/navbar';
import { Sidebar } from '@/components/sidebar';
import { source } from '@/lib/source';

export default function Layout({ children }: LayoutProps<'/docs'>) {
  return (
    <DocsLayout
      tree={source.pageTree}
      nav={{ component: <DocsNavBar /> }}
      sidebar={{ collapsible: false, className: 'bg-fd-background md:-mt-6', component: <Sidebar /> }}
      searchToggle={{ enabled: false }}
      themeSwitch={{ enabled: false }}
    >
      {children}
    </DocsLayout>
  );
}
