import React, { useState } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import { 
  Play, 
  Cpu, 
  ShieldCheck, 
  MonitorSmartphone, 
  Database, 
  Sparkles, 
  Terminal, 
  Flame, 
  Layers, 
  Radio, 
  Server,
  Laptop,
  Box,
  Copy,
  Check,
  ChevronRight 
} from 'lucide-react';

function HeroSection() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className="hero-wrapper">
      <div className="hero-beam-bg" />
      <div className="hero-content">
        <div className="hero-logo-container">
          <img 
            src="img/app-icon.svg" 
            alt="Kinema App Icon" 
            className="hero-projector-logo" 
            style={{ width: '100px', height: '100px', borderRadius: '24px', boxShadow: '0 8px 32px rgba(232, 85, 58, 0.4)' }}
          />
        </div>

        <div className="hero-badges-row">
          <span className="kinema-badge">
            <Flame size={14} /> Rust 2024 Edition
          </span>
          <span className="kinema-badge kinema-badge-gold">
            <ShieldCheck size={14} /> 100% Clean-Room
          </span>
          <span className="kinema-badge">
            <Radio size={14} /> Multi-Platform
          </span>
        </div>

        <Heading as="h1" className="hero-main-title">
          {siteConfig.title}
        </Heading>

        <p className="hero-main-subtitle">
          High-performance, self-hosted personal media server in Rust. Direct-play prioritization, throttled fMP4 transcoding, zero runtime bloat, and native clients across Web, Desktop, and Android.
        </p>

        <div className="hero-actions-row">
          <Link
            className="btn-cinema-primary"
            to="/docs/getting-started/overview">
            Get Started <ChevronRight size={16} />
          </Link>
          <Link
            className="btn-cinema-secondary"
            to="/docs/architecture/system-overview">
            <Layers size={16} /> Architecture Specs
          </Link>
          <Link
            className="btn-cinema-secondary"
            href="https://github.com/aiyu-ayaan/Kinema">
            GitHub
          </Link>
        </div>
      </div>
    </header>
  );
}

const FeatureList = [
  {
    title: 'Direct Play First & Remux Engine',
    Icon: Play,
    description:
      'Always checks client decoder capabilities before touching a frame. Ranged sendfile for container matches, zero-copy fMP4 stream copying for codec matches. Full video transcoding is strictly the last resort.',
    link: '/docs/media-pipeline/playback-engine',
  },
  {
    title: 'Throttled & Sandboxed Transcoding',
    Icon: Cpu,
    description:
      'Re-encodes are actively paced 60 seconds ahead of real-time playback at nice 10 (nice 19 for background jobs). Low-power CPUs never spike to 100% all-core saturation; child processes terminate instantly on disconnect.',
    link: '/docs/media-pipeline/ffmpeg-execution',
  },
  {
    title: 'Clean-Room Engineering',
    Icon: ShieldCheck,
    description:
      'Zero decompiled proprietary code from Plex or closed-source platforms. Built strictly on open media standards (Matroska, MP4, WebVTT, HLS) and public TMDB / TVmaze metadata APIs with full legal compliance.',
    link: '/docs/architecture/clean-room',
  },
  {
    title: 'Multi-Platform Native Ecosystem',
    Icon: MonitorSmartphone,
    description:
      'Unified media experience across React 18 Web SPA (with Web Audio 200% booster), Electron Desktop shell (with mpv loopback proxy for zero-transcode desktop playback), and Android Jetpack Compose with ExoPlayer.',
    link: '/docs/clients/overview',
  },
  {
    title: 'SQLite WAL & Zero-Config Storage',
    Icon: Database,
    description:
      'No complex database services to maintain. High-concurrency SQLite with Write-Ahead Logging (WAL), FTS5 instant search, per-profile track choices, and automated schema migrations.',
    link: '/docs/architecture/data-model',
  },
  {
    title: 'Audio Fingerprint Theme Detection',
    Icon: Sparkles,
    description:
      'Automated intro and credit skipping using 33-band Haitsma & Kalker audio fingerprinting, ffprobe chapter titles, and keyframe black-frame detectors. Seamlessly skips openings on web and Android.',
    link: '/docs/media-pipeline/intro-credit-detection',
  },
];

function FeaturesSection() {
  return (
    <section className="section-wrapper">
      <div className="section-header">
        <h2 className="section-title">Architected for Speed & Efficiency</h2>
        <p className="section-subtitle">
          Engineered from scratch to stream massive 4K HDR libraries from low-power devices like Raspberry Pi 5 without thermal throttling.
        </p>
      </div>

      <div className="cards-grid">
        {FeatureList.map((props, idx) => (
          <div key={idx} className="kinema-card">
            <div>
              <div className="card-icon-wrapper">
                <props.Icon size={24} />
              </div>
              <h3 className="card-title">{props.title}</h3>
              <p className="card-text">{props.description}</p>
            </div>
            <Link to={props.link} className="card-link">
              Learn more <ChevronRight size={14} />
            </Link>
          </div>
        ))}
      </div>
    </section>
  );
}

function QuickStartSection() {
  const [tab, setTab] = useState<'server' | 'desktop' | 'docker'>('server');
  const [copied, setCopied] = useState(false);

  const snippets = {
    server: `# Home Server (Linux x86_64 / aarch64) - Standalone bundle with backend, web UI, bundled FFmpeg 8.1 & systemd service
curl -fsSL https://raw.githubusercontent.com/aiyu-ayaan/Kinema/master/scripts/install-server.sh | sudo sh`,
    desktop: `# Desktop App (Linux x86_64) - AppImage, desktop application menu entry with icon & 'kinema' CLI
curl -fsSL https://raw.githubusercontent.com/aiyu-ayaan/Kinema/master/scripts/install-desktop.sh | sh`,
    docker: `# Run with Docker Compose
wget https://raw.githubusercontent.com/aiyu-ayaan/Kinema/master/docker-compose.yml
MEDIA_DIR=/path/to/media docker compose up -d`,
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(snippets[tab]);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section style={{ backgroundColor: 'rgba(232, 85, 58, 0.04)', padding: '4.5rem 1.5rem', borderTop: '1px solid var(--kinema-border)' }}>
      <div className="code-box-container">
        <div className="section-header" style={{ marginBottom: '2rem' }}>
          <h2 className="section-title">Quick Install</h2>
          <p className="section-subtitle">Install the native Kinema server bundle or desktop app with a single terminal command.</p>
        </div>

        {/* Tab Buttons */}
        <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem', flexWrap: 'wrap' }}>
          <button
            onClick={() => setTab('server')}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '0.4rem',
              padding: '0.6rem 1.2rem',
              borderRadius: '8px',
              border: '1px solid var(--kinema-border)',
              backgroundColor: tab === 'server' ? 'var(--kinema-ember)' : 'var(--kinema-card-bg)',
              color: tab === 'server' ? '#fff' : 'var(--ifm-heading-color)',
              fontWeight: 600,
              fontSize: '0.9rem',
              cursor: 'pointer',
              transition: 'all 0.15s ease',
            }}>
            <Server size={16} /> Home Server (Recommended)
          </button>
          <button
            onClick={() => setTab('desktop')}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '0.4rem',
              padding: '0.6rem 1.2rem',
              borderRadius: '8px',
              border: '1px solid var(--kinema-border)',
              backgroundColor: tab === 'desktop' ? 'var(--kinema-ember)' : 'var(--kinema-card-bg)',
              color: tab === 'desktop' ? '#fff' : 'var(--ifm-heading-color)',
              fontWeight: 600,
              fontSize: '0.9rem',
              cursor: 'pointer',
              transition: 'all 0.15s ease',
            }}>
            <Laptop size={16} /> Desktop App (Linux)
          </button>
          <button
            onClick={() => setTab('docker')}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '0.4rem',
              padding: '0.6rem 1.2rem',
              borderRadius: '8px',
              border: '1px solid var(--kinema-border)',
              backgroundColor: tab === 'docker' ? 'var(--kinema-ember)' : 'var(--kinema-card-bg)',
              color: tab === 'docker' ? '#fff' : 'var(--ifm-heading-color)',
              fontWeight: 600,
              fontSize: '0.9rem',
              cursor: 'pointer',
              transition: 'all 0.15s ease',
            }}>
            <Box size={16} /> Docker Compose
          </button>
        </div>

        <div className="code-box-card">
          <div className="code-box-header" style={{ justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <Terminal size={18} color="#e8553a" />
              <span className="code-box-badge">
                {tab === 'server' ? 'install-server.sh' : tab === 'desktop' ? 'install-desktop.sh' : 'docker-compose'}
              </span>
            </div>
            <button
              onClick={handleCopy}
              style={{
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                color: copied ? '#4ade80' : 'var(--ifm-heading-color)',
                display: 'flex',
                alignItems: 'center',
                gap: '0.3rem',
                fontSize: '0.82rem',
                fontWeight: 600,
              }}>
              {copied ? <Check size={14} /> : <Copy size={14} />}
              {copied ? 'Copied!' : 'Copy Command'}
            </button>
          </div>
          <pre className="code-box-pre">
            {snippets[tab]}
          </pre>
          <div className="code-box-footer">
            <span style={{ fontSize: '0.85rem', opacity: 0.8 }}>
              {tab === 'server'
                ? 'Installs service to /opt/kinema with bundled FFmpeg 8.1 & systemd auto-restart'
                : tab === 'desktop'
                ? 'Installs AppImage to ~/.local/share/kinema with app icon & mpv zero-transcode player'
                : 'Runs multi-arch image on port 3000 with host networking for UDP discovery'}
            </span>
            <Link to="/docs/getting-started/quick-start" className="card-link">
              Read install options & flags <ChevronRight size={14} />
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title="Personal Media Server in Rust"
      description={siteConfig.tagline}>
      <HeroSection />
      <main>
        <FeaturesSection />
        <QuickStartSection />
      </main>
    </Layout>
  );
}
