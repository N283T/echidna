// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/installation',
        'getting-started/quick-start',
        'getting-started/requirements',
      ],
    },
    {
      type: 'category',
      label: 'Commands',
      collapsed: false,
      items: [
        'commands/init',
        'commands/build',
        'commands/install',
        'commands/run',
        'commands/test',
        'commands/watch',
        'commands/debug',
        'commands/venv',
        'commands/clean',
        'commands/validate',
        'commands/info',
        'commands/python',
        'commands/packages',
        'commands/version',
        'commands/workspace',
        'commands/docs',
        'commands/publish',
        'commands/completions',
      ],
    },
    {
      type: 'category',
      label: 'Configuration',
      items: [
        'configuration/echidna-toml',
        'configuration/chimerax-detection',
        'configuration/project-structure',
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/bundle-types',
      ],
    },
    'contributing',
  ],
};

export default sidebars;
