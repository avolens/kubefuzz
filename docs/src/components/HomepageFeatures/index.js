import React from 'react';
import clsx from 'clsx';
import styles from './styles.module.css';

const FeatureList = [
  {
    title: 'Test complex Admission Controller chains',
    Svg: require('@site/static/img/peer-to-peer.svg').default,
    description: (
      <>
       Configure resource generation using a simple constraint
       language and let KubeFuzz automatically uncover unexpected behavior in your
       cluster.
      </>
    ),
  },
  {
    title: 'Supports all your resources',
    Svg: require('@site/static/img/resources.svg').default,
    description: (
      <>
        KubeFuzz automatically understands your custom resource definitions and pulls
        them from the cluser API to generate semantically correct test cases.
      </>
    ),
  },
  {
    title: 'Open Source',
    Svg: require('@site/static/img/configure.svg').default,
    description: (
      <>
       Build in Rust and open sourced under the Apache 2.0 license, KubeFuzz benefits
       from the Kubernetes open source community.
      </>
    ),
  },
];

function Feature({Svg, title, description}) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <h3>{title}</h3>
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
