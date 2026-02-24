import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

const FeatureList = [
  {
    title: 'Quick Scaffolding',
    description: (
      <>
        Generate a complete ChimeraX bundle project with one command.
        Supports 8 bundle types: command, tool, format, fetch, and more.
      </>
    ),
  },
  {
    title: 'Fast Iteration',
    description: (
      <>
        Build, install, and launch ChimeraX in a single command.
        Watch mode auto-rebuilds on file changes.
      </>
    ),
  },
  {
    title: 'IDE Integration',
    description: (
      <>
        Set up virtual environments for type checkers and autocompletion.
        Works with ty, ruff, pyright, and any LSP-based editor.
      </>
    ),
  },
  {
    title: 'Package Management',
    description: (
      <>
        Install, check, and list packages in ChimeraX's Python environment.
        Uses uv for speed when available.
      </>
    ),
  },
  {
    title: 'Testing Built In',
    description: (
      <>
        Run pytest tests and smoke tests through ChimeraX's Python.
        Coverage reporting included.
      </>
    ),
  },
  {
    title: 'Cross-Platform',
    description: (
      <>
        Works on macOS, Linux, and Windows.
        Auto-detects ChimeraX installation on all platforms.
      </>
    ),
  },
];

function Feature({title, description}) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md padding-vert--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
