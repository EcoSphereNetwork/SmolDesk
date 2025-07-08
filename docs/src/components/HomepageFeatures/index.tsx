import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

/**
 * Single card information used on the landing page.
 * Add more entries to FeatureList below to extend the section.
 */
type FeatureItem = {
  title: string;
  icon: ReactNode; // Svg component or emoji
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Remote-Desktop-Zugriff',
    icon: (
      <img
        src={require('@site/static/img/undraw_docusaurus_mountain.svg').default}
        className={styles.featureSvg}
        alt="Remote Desktop"
      />
    ),
    description: <>Greife von überall auf deinen Linux-PC zu und erhalte das Bild nahezu in Echtzeit.</>,
  },
  {
    title: 'Sichere Peer-to-Peer Verbindung',
    icon: (
      <img
        src={require('@site/static/img/undraw_docusaurus_tree.svg').default}
        className={styles.featureSvg}
        alt="Sichere Verbindung"
      />
    ),
    description: <>SmolDesk verbindet die Geräte direkt miteinander. Signaling läuft verschlüsselt.</>,
  },
  {
    title: 'Mobile-optimierte Oberfläche',
    icon: (
      <img
        src={require('@site/static/img/undraw_docusaurus_react.svg').default}
        className={styles.featureSvg}
        alt="Mobile Oberfläche"
      />
    ),
    description: <>An kleine Bildschirme angepasst und mit Gestensteuerung sowie Darkmode ausgestattet.</>,
  },
  {
    title: 'Plattformübergreifend',
    icon: (
      <img
        src={require('@site/static/img/undraw_docusaurus_tree.svg').default}
        className={styles.featureSvg}
        alt="Plattformübergreifend"
      />
    ),
    description: <>Nutze SmolDesk unter Linux und Android. Eine iOS-Version ist in Planung.</>,
  },
  {
    title: 'Modular erweiterbar',
    icon: (
      <img
        src={require('@site/static/img/undraw_docusaurus_mountain.svg').default}
        className={styles.featureSvg}
        alt="Modular"
      />
    ),
    description: <>Funktionen lassen sich über optionale Module jederzeit ergänzen.</>,
  },
];

function Feature({title, icon, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        {icon}
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
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
