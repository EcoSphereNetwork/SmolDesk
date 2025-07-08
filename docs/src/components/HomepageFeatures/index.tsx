import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

/**
 * Single card information used on the landing page.
 * Add more entries to FeatureList below to extend the section.
 */
type FeatureItem = {
  title: string;
  to: string;
  icon: string; // svg path
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: '🖥️ Remote Desktop',
    to: '/features/remote',
    description: <>Greife von überall auf deinen Linux-PC zu.</>,
  },
  {
    title: '🔒 Sicherheit',
    to: '/features/security',
    description: <>Ende-zu-Ende-Verschlüsselung für alle Daten.</>,
  },
  {
    title: '📋 ClipboardSync',
    to: '/components/ClipboardSync',
    description: <>Teile Text und Bilder bequem zwischen Geräten.</>,
  },
  {
    title: '📂 Dateiübertragung',
    to: '/components/FileTransfer',
    description: <>Sende Dateien per Drag & Drop.</>,
  },
  {
    title: '🖥️🖥️ Multi-Monitor',
    to: '/features/monitors',
    description: <>Wähle im Viewer den gewünschten Bildschirm.</>,
  },
  {
    title: '📱 Mobile App',
    to: '/development/setup-android',
    description: <>Volle Kontrolle auch unterwegs.</>,
  },
  {
    title: '🧪 Playground',
    to: '/demo/live-demo',
    description: <>Teste Komponenten und APIs direkt im Browser.</>,
  },
];

function Feature({title, to, icon, description}: FeatureItem) {
  return (
    <Link className={styles.card} to={to}>
      <img src={icon} className={styles.icon} alt="" />
      <Heading as="h3" className={styles.title}>{title}</Heading>
      <p>{description}</p>
    </Link>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className={styles.grid}>
        {FeatureList.map((props, idx) => (
          <Feature key={idx} {...props} />
        ))}
      </div>
    </section>
  );
}
