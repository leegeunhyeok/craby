import type { PropsWithChildren } from 'react';

export interface FeatureProps {
  title: string;
  emoji: string;
}

export function Feature({ title, emoji, children }: PropsWithChildren<FeatureProps>) {
  return (
    <div className="flex cursor-default flex-col gap-2 rounded-xl border bg-fd-card p-4 text-left text-fd-card-foreground transition-all duration-300 hover:border-fd-primary hover:bg-fd-secondary sm:gap-4 sm:p-8">
      <p className="font-semibold text-md sm:text-lg">
        <span className="tossface mr-2">{emoji}</span>
        {title}
      </p>
      <p className="whitespace-pre-wrap text-left text-gray-500 text-sm sm:text-base">{children}</p>
    </div>
  );
}
