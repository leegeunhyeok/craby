'use client';

import { type AnnotationHandler, type HighlightedCode, highlight, InnerToken, Pre } from 'codehike/code';
import dedent from 'dedent';
import { cn } from 'fumadocs-ui/utils/cn';
import { use, useEffect, useMemo, useRef, useState } from 'react';
import { RustIcon } from './icons/rust';
import { TypeScriptIcon } from './icons/typescript';
import { SmoothPre } from './smooth-pre';

interface Code {
  lang: string;
  code: string;
}

const tokenTransitions: AnnotationHandler = {
  name: 'token-transitions',
  PreWithRef: SmoothPre,
  Token: (props) => <InnerToken merge={props} style={{ display: 'inline-block' }} />,
};

const PREVIEW_INTERVAL = 3_000;
const DEMO_CODE = getDemoCode();

const handlers = [tokenTransitions];

export function CodePreview() {
  const isHiddenRef = useRef<boolean>(false);
  const highlighted = useHighlight(DEMO_CODE);
  const [index, setIndex] = useState(0);

  useEffect(() => {
    const handleVisibilityChange = () => {
      isHiddenRef.current = document.hidden;
    };

    window.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      window.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
      if (isHiddenRef.current) {
        return;
      }
      setIndex((prev) => (prev === highlighted.length - 1 ? 0 : prev + 1));
    }, PREVIEW_INTERVAL);

    return () => clearInterval(interval);
  }, [highlighted]);

  if (highlighted[index] == null) return null;

  return <CodePreviewWindow code={highlighted[index]} lang={highlighted[index]?.lang} />;
}

export function CodePreviewFallback() {
  return <CodePreviewWindow code={null} lang="typescript" />;
}

interface CodePreviewWindowProps {
  code: HighlightedCode | null;
  lang: string;
}

function CodePreviewWindow({ code, lang }: CodePreviewWindowProps) {
  return (
    <div className="flex w-full flex-col">
      <div className="flex overflow-hidden rounded-lg rounded-b-none bg-[#303030]">
        <div
          className={cn(
            'flex items-center gap-2 px-3 py-2 text-[#777] transition-[background-color,color] duration-300 md:px-5 md:py-3',
            lang === 'typescript' && 'bg-[#1e1e1e] text-white',
          )}
        >
          <TypeScriptIcon className="h-3 xs:h-4 w-3 xs:w-4 fill-[#3178c6]" />
          <p className="text-[10px] xs:text-sm">TypeScript</p>
        </div>
        <div
          className={cn(
            'flex items-center gap-2 px-3 py-2 text-[#777] transition-[background-color,color] duration-300 md:px-5 md:py-3',
            lang === 'rust' && 'bg-[#1e1e1e] text-white',
          )}
        >
          <RustIcon className="h-3 xs:h-4 w-3 xs:w-4 fill-[#d34516]" />
          <p className="text-[10px] xs:text-sm">Rust</p>
        </div>
      </div>
      <div className="min-h-[15em] xs:min-h-[16em] rounded-lg rounded-t-none bg-[#1e1e1e] p-3 xs:p-4 text-left text-[11px] xs:text-[13px]">
        {code && (
          <Pre
            code={code}
            handlers={handlers}
            className="overflow-x-auto overflow-y-hidden bg-transparent text-left text-[1em]"
          />
        )}
      </div>
    </div>
  );
}

function useHighlight(codes: Code[]) {
  const highlightTasks = useMemo(
    () => Promise.all(codes.map(({ code, lang }) => highlight({ value: code, lang, meta: '' }, 'dark-plus'))),
    [codes],
  );

  return use(highlightTasks);
}

function getDemoCode() {
  let maxLineWidth = 0;

  const RUST_DEMO_CODE = dedent`impl CalculatorSpec for Calculator {
      fn add(&mut self: a: Number, b: Number) -> Number {
          a + b
      }

      fn sub(&mut self: a: Number, b: Number) -> Number {
          a - b
      }
  }`;

  RUST_DEMO_CODE.split('\n').forEach((line) => {
    maxLineWidth = Math.max(maxLineWidth, line.length);
  });

  const TYPESCRIPT_DEMO_CODE = dedent`interface CalculatorSpec extends NativeModule {
    add(a: number, b: number): number;
    sub(a: number, b: number): number;
  }

  NativeModuleRegistry.getEnforcing<CalculatorSpec>(
    'Calculator',
  );`;

  return [
    {
      lang: 'typescript',
      code: `${TYPESCRIPT_DEMO_CODE}\n${' '.repeat(maxLineWidth)}`,
    },
    {
      lang: 'rust',
      code: RUST_DEMO_CODE,
    },
  ];
}
