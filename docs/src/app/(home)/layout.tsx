import { HomeNavBar } from '@/components/navbar';

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <div>
      <HomeNavBar />
      <main className="pt-[56px]">{children}</main>
    </div>
  );
}
