import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

const readerPaths = [
  {
    label: '01 // Understand',
    title: 'Start with the runtime boundary',
    description: 'Learn why agents propose work while trusted hosts own side effects, policy, persistence, and credentials.',
    to: '/docs/architecture/intro',
  },
  {
    label: '02 // Configure',
    title: 'Choose a supported profile',
    description: 'Configure server-full, minimal, or embedded-mobile without transferring claims across profiles.',
    to: '/docs/configuration',
  },
  {
    label: '03 // Build',
    title: 'Create an agent workflow',
    description: 'Connect models, tools, skills, knowledge, memory, and approvals through one runtime contract.',
    to: '/docs/intro',
  },
  {
    label: '04 // Integrate',
    title: 'Use typed protocol surfaces',
    description: 'Enter through HTTP, MCP, A2A, AG-UI, A2UI, or a supported SDK while UAR keeps execution coherent.',
    to: '/docs/api',
  },
] as const;

const surfaceSteps = [
  ['Canvas', 'Runtime intent and user context'],
  ['Chrome', 'Navigation, tenancy, and policy'],
  ['Surface', 'Turns, steps, tools, and retrieval'],
  ['Raised', 'Evidence, approvals, and results'],
] as const;

const protocols = ['OpenAI-compatible', 'Anthropic-compatible', 'MCP', 'A2A', 'AG-UI', 'A2UI'] as const;

export default function Home(): ReactNode {
  return (
    <Layout
      title="Governed Agent Execution"
      description="Universal Agent Runtime unifies models, tools, skills, knowledge, memory, policy, and typed agent protocols behind one trusted execution boundary.">
      <main className={styles.page}>
        <section className={styles.hero} aria-labelledby="uar-home-title">
          <div className={styles.heroCopy}>
            <p className={styles.eyebrow}>Prometheus AGS // Runtime Infrastructure</p>
            <Heading as="h1" id="uar-home-title" className={styles.title}>
              One trusted boundary for agent execution.
            </Heading>
            <p className={styles.lede}>
              UAR coordinates models, tools, skills, knowledge, memory, policy,
              and streaming protocols without giving the agent kernel authority
              to mutate the world.
            </p>
            <div className={styles.actions}>
              <Link className="button button--primary button--lg" to="/docs/intro">
                Read the Documentation
              </Link>
              <Link className="button button--secondary button--lg" to="/docs/installation">
                Install UAR
              </Link>
            </div>
          </div>
          <div className={styles.brandField} aria-hidden="true">
            <img className={styles.wordmarkDark} src="img/brand/uar-wordmark-dark.svg" alt="" width="520" height="96" />
            <img className={styles.wordmarkLight} src="img/brand/uar-wordmark-light.svg" alt="" width="520" height="96" />
            <div className={styles.runtimeReadout}>
              <span>HOST</span><strong>trusted</strong>
              <span>KERNEL</span><strong>proposal-only</strong>
              <span>EVENTS</span><strong>typed</strong>
              <span>POLICY</span><strong>fail-closed</strong>
            </div>
          </div>
        </section>

        <section className={styles.section} aria-labelledby="boundary-title">
          <div className={styles.sectionHeading}>
            <p className={styles.eyebrow}>Execution Contract</p>
            <Heading as="h2" id="boundary-title">Capability inversion is the safety boundary.</Heading>
          </div>
          <div className={styles.boundary}>
            <article className={styles.boundaryPanel}>
              <span className={styles.signal}>Agent kernel</span>
              <Heading as="h3">Reason. Select. Propose.</Heading>
              <p>The kernel creates intent and structured requests. It does not hold write authority.</p>
            </article>
            <div className={styles.boundaryFlow} aria-hidden="true">request → policy → event</div>
            <article className={styles.boundaryPanelStrong}>
              <span className={styles.signal}>Trusted host</span>
              <Heading as="h3">Authorize. Execute. Record.</Heading>
              <p>The host enforces tenancy, credentials, governance, persistence, and observable effects.</p>
            </article>
          </div>
        </section>

        <section className={styles.section} aria-labelledby="surface-title">
          <div className={styles.sectionHeading}>
            <p className={styles.eyebrow}>Four-Step Surface Ladder</p>
            <Heading as="h2" id="surface-title">From intent to evidence, every layer stays inspectable.</Heading>
          </div>
          <div className={styles.surfaceGrid}>
            {surfaceSteps.map(([name, description], index) => (
              <article className={styles[`surface${index + 1}`]} key={name}>
                <span>{String(index + 1).padStart(2, '0')}</span>
                <Heading as="h3">{name}</Heading>
                <p>{description}</p>
              </article>
            ))}
          </div>
        </section>

        <section className={styles.protocolSection} aria-labelledby="protocol-title">
          <div>
            <p className={styles.eyebrow}>Open Protocols First</p>
            <Heading as="h2" id="protocol-title">One runtime, multiple typed entrances.</Heading>
            <p className={styles.sectionCopy}>Protocols describe how work crosses the boundary. They do not bypass policy, identity, or lifecycle control.</p>
          </div>
          <ul className={styles.protocolList} aria-label="Supported protocol families">
            {protocols.map((protocol) => <li key={protocol}>{protocol}</li>)}
          </ul>
        </section>

        <section className={styles.section} aria-labelledby="paths-title">
          <div className={styles.sectionHeading}>
            <p className={styles.eyebrow}>Choose Your Path</p>
            <Heading as="h2" id="paths-title">Move from theory to a working boundary.</Heading>
          </div>
          <div className={styles.pathGrid}>
            {readerPaths.map((path) => (
              <Link className={styles.pathCard} to={path.to} key={path.label}>
                <span>{path.label}</span>
                <Heading as="h3">{path.title}</Heading>
                <p>{path.description}</p>
                <strong>Open guide →</strong>
              </Link>
            ))}
          </div>
        </section>

        <section className={styles.profileNote} aria-labelledby="profiles-title">
          <div>
            <p className={styles.eyebrow}>Evidence Has Boundaries</p>
            <Heading as="h2" id="profiles-title">Profiles are separate contracts.</Heading>
          </div>
          <p><strong>server-full</strong>, <strong>minimal</strong>, and <strong>embedded-mobile</strong> differ in capability and evidence. A result for one never transfers silently to another.</p>
          <Link to="/docs/configuration">Compare configuration boundaries →</Link>
        </section>
      </main>
    </Layout>
  );
}
