import { CrabyDocsLayout } from '@/components/docs-layout';
import { DocsNavBar } from '@/components/navbar';
import { source } from '@/lib/source';

export default function Layout({ children }: LayoutProps<'/docs'>) {
  return (
    <CrabyDocsLayout
      tree={source.pageTree}
      nav={{ component: <DocsNavBar /> }}
      sidebar={{ collapsible: false, className: 'bg-fd-background md:-mt-6' }}
      searchToggle={{ enabled: false }}
      themeSwitch={{ enabled: false }}
    >
      {children}
    </CrabyDocsLayout>
  );
}
