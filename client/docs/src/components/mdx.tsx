import defaultMdxComponents from 'fumadocs-ui/mdx';
import type { MDXComponents } from 'mdx/types';
import { Card, Cards } from './card';
import { TypeTable } from './type-table';
import { Callout } from './callout';
import { Tabs, Tab } from './tabs';
import { Steps, Step } from './steps';
import { Accordion, Accordions } from './accordion';

export function getMDXComponents(components?: MDXComponents) {
  return {
    ...defaultMdxComponents,
    Card,
    Cards,
    TypeTable,
    Callout,
    Tabs,
    Tab,
    Steps,
    Step,
    Accordion,
    Accordions,
    ...components,
  } satisfies MDXComponents;
}

export const useMDXComponents = getMDXComponents;

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}

