import { baseOptions } from '@/lib/layout.shared';
import { HomeLayout } from 'fumadocs-ui/layouts/home';
import Link from 'next/link';

export default function NotFound() {
  return (
    <HomeLayout {...baseOptions('home')}>
      <div className="container py-12 text-center">
        <h1 className="text-6xl font-bold mb-4">404</h1>
        <p className="text-xl text-muted-foreground mb-8">페이지를 찾을 수 없습니다</p>
        <div className="flex gap-4 justify-center">
          <Link
            href="/docs/get-started/introduction"
            className="px-6 py-2 rounded-lg bg-primary text-primary-foreground"
          >
            문서 보기
          </Link>
          <Link href="/" className="px-6 py-2 rounded-lg border">
            홈으로
          </Link>
        </div>
      </div>
    </HomeLayout>
  );
}
