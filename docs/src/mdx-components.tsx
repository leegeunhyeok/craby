import { Mermaid } from '@/components/mdx/mermaid';
import { TossFace } from '@/components/mdx/tossface';
import defaultMdxComponents from 'fumadocs-ui/mdx';
import * as TabsComponents from 'fumadocs-ui/components/tabs';
import type { MDXComponents } from 'mdx/types';
import { CodeBlock, Pre } from 'fumadocs-ui/components/codeblock';

export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    ...defaultMdxComponents,
    ...TabsComponents,
    ...components,
    Mermaid,
    TossFace,
    Callout: (props) => <defaultMdxComponents.Callout {...props} className="shadow-none border-none pl-0" />,
    pre: ({ ref: _ref, ...props }) => (
      <CodeBlock {...props} className="shadow-none">
        <Pre>{props.children}</Pre>
      </CodeBlock>
    ),
  };
}
