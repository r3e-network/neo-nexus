export type DocsNavigationItem = {
  label: string;
  anchor: string;
};

export type DocsNavigationSection = {
  title: string;
  items: DocsNavigationItem[];
};

export const docsNavigation: DocsNavigationSection[] = [
  {
    title: 'Getting Started',
    items: [
      { label: 'Introduction', anchor: '#introduction' },
      { label: 'Create an Account', anchor: '#create-account' },
      { label: 'First API Request', anchor: '#quick-start' },
    ],
  },
  {
    title: 'Nodes',
    items: [
      { label: 'Shared Endpoints', anchor: '#shared-endpoints' },
      { label: 'Dedicated Nodes', anchor: '#dedicated-nodes' },
      { label: 'WebSockets (WSS)', anchor: '#websockets' },
    ],
  },
  {
    title: 'Managed Plugins',
    items: [
      { label: 'TEE Oracle', anchor: '#tee-oracle' },
      { label: 'AA Bundler', anchor: '#aa-bundler' },
    ],
  },
];

export function filterDocsNavigation(
  sections: DocsNavigationSection[],
  query: string,
) {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) {
    return sections;
  }

  return sections
    .map((section) => ({
      ...section,
      items: section.items.filter((item) => item.label.toLowerCase().includes(normalizedQuery)),
    }))
    .filter((section) => section.items.length > 0);
}
