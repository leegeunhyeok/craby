import { Suspense } from 'react';
import { CodePreview } from '@/components/code-preview';

export default function HomePage() {
  return (
    <div className="flex max-w-[1200px] flex-1 flex-col p-4 pt-16 text-center lg:pt-20">
      <section className="flex flex-row items-center justify-center max-[1100px]:flex-col max-[1100px]:gap-14">
        <div className="flex max-w-[600px] flex-col items-start justify-center whitespace-pre-wrap text-left max-[1100px]:items-center">
          <p className="w-fit bg-[linear-gradient(120deg,#82d7f7_35%,#387ca0)] bg-clip-text font-bold text-4xl text-transparent leading-12 tracking-tight antialiased max-[1100px]:text-center sm:text-5xl sm:leading-15 md:text-6xl md:leading-18">
            Craby
          </p>
          <p className="leading:10 sm:leading:12 font-bold text-4xl text-fd-foreground-secondary tracking-tight antialiased max-[1100px]:text-center sm:text-5xl md:text-6xl md:leading-15">
            Type-safe Rust for React Native
          </p>
          <p className="mt-2 text-fd-muted-foreground text-lg max-[1100px]:text-center md:mt-4 md:text-2xl">
            Auto generated, integrated with pure C++ TurboModule
          </p>
        </div>
        <div className="flex w-full max-w-[600px] pl-8 drop-shadow-[0_0_25px_rgba(130,215,247,0.8)] max-[1100px]:pl-0">
          <Suspense fallback={null}>
            <CodePreview />
          </Suspense>
        </div>
      </section>
    </div>
  );
}
