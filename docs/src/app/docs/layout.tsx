import { source } from '@/lib/source';
import { DocsLayout } from 'fumadocs-ui/layouts/docs';
import { baseOptions } from '@/lib/layout.shared';

export default function Layout({ children }: LayoutProps<'/docs'>) {
  return (
    <DocsLayout
      tree={source.pageTree}
      {...baseOptions('docs')}
      nav={{ enabled: false }}
      sidebar={{ collapsible: false, className: 'bg-fd-background' }}
    >
      {children}
    </DocsLayout>
  );
}
