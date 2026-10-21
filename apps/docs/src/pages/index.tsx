import React from 'react';
import clsx from 'clsx';
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
  ChevronRight 
} from 'lucide-react';

function HeroSection() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className="hero hero--dark relative overflow-hidden text-center py-20 px-4" style={{ minHeight: '520px', display: 'flex', alignItems: 'center' }}>
      <div className="hero-beam-bg" />
      <div className="container relative z-10 max-w-5xl mx-auto">
        <div className="flex justify-center mb-6">
          <div className="relative">
            <img 
              src="img/logo.svg" 
              alt="Kinema Projector K" 
              className="hero-projector-logo w-24 h-24 sm:w-28 sm:h-28" 
            />
          </div>
        </div>

        <div className="flex flex-wrap items-center justify-center gap-2 mb-4">
          <span className="kinema-badge">
            <Flame className="w-3.5 h-3.5 mr-1" /> Rust 2024 Edition
          </span>
          <span className="kinema-badge kinema-badge-gold">
            <ShieldCheck className="w-3.5 h-3.5 mr-1" /> 100% Clean-Room
          </span>
          <span className="kinema-badge">
            <Radio className="w-3.5 h-3.5 mr-1" /> Multi-Platform Clients
          </span>
        </div>

        <Heading as="h1" className="hero__title text-4xl sm:text-6xl font-extrabold tracking-tight mb-4">
          {siteConfig.title}
        </Heading>

        <p className="hero__subtitle text-lg sm:text-xl max-w-2xl mx-auto opacity-90 font-normal mb-8">
          A high-performance personal media server built in Rust. Direct-play prioritization, throttled fMP4 transcoding, zero runtime bloat, and native clients across Web, Desktop, and Android.
        </p>

        <div className="flex flex-wrap items-center justify-center gap-4">
          <Link
            className="button button--primary button--lg flex items-center font-semibold px-6 py-3"
            style={{ backgroundColor: 'var(--kinema-ember)', borderColor: 'var(--kinema-ember)', color: '#ffffff' }}
            to="/docs/getting-started/overview">
            Get Started <ChevronRight className="w-4 h-4 ml-1 inline" />
          </Link>
          <Link
            className="button button--secondary button--lg flex items-center font-semibold px-6 py-3"
            to="/docs/architecture/system-overview">
            <Layers className="w-4 h-4 mr-1.5 inline" /> Architecture Specs
          </Link>
          <Link
            className="button button--outline button--secondary button--lg font-semibold px-6 py-3"
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
      'Unified media experience across React 18 Web SPA (with Web Audio 200% booster), Electron Desktop shell (with mpv loopback proxy for zero-transcode playback), and Android Jetpack Compose with ExoPlayer.',
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
    <section className="py-16 px-4">
      <div className="container max-w-6xl mx-auto">
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold tracking-tight mb-3">Architected for Speed & Efficiency</h2>
          <p className="text-lg opacity-80 max-w-2xl mx-auto">
            Engineered from scratch to stream massive 4K HDR libraries from low-power devices like Raspberry Pi 5 without thermal throttling.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {FeatureList.map((props, idx) => (
            <div key={idx} className="kinema-card flex flex-col justify-between">
              <div>
                <div 
                  className="w-12 h-12 rounded-lg flex items-center justify-center mb-4" 
                  style={{ backgroundColor: 'rgba(232, 85, 58, 0.15)', color: 'var(--kinema-ember)' }}
                >
                  <props.Icon className="w-6 h-6" />
                </div>
                <h3 className="text-xl font-semibold mb-2">{props.title}</h3>
                <p className="text-sm opacity-85 leading-relaxed mb-4">{props.description}</p>
              </div>
              <Link 
                to={props.link} 
                className="text-sm font-semibold flex items-center" 
                style={{ color: 'var(--kinema-ember)' }}
              >
                Learn more <ChevronRight className="w-3.5 h-3.5 ml-1" />
              </Link>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function QuickStartSection() {
  return (
    <section className="py-16 px-4" style={{ backgroundColor: 'rgba(232, 85, 58, 0.03)' }}>
      <div className="container max-w-4xl mx-auto">
        <div className="text-center mb-8">
          <h2 className="text-3xl font-bold tracking-tight mb-2">Deploy in Seconds</h2>
          <p className="opacity-80">Get Kinema running locally with Docker or build natively with Cargo.</p>
        </div>

        <div className="kinema-card p-6">
          <div className="flex items-center gap-2 mb-3 pb-3 border-b border-gray-700">
            <Terminal className="w-4 h-4 text-orange-400" />
            <span className="text-xs font-mono font-bold uppercase tracking-wider opacity-75">docker-compose.yml</span>
          </div>
          <pre className="text-sm font-mono p-4 rounded overflow-x-auto bg-black/40 text-gray-200">
{`services:
  kinema:
    image: ghcr.io/aiyu-ayaan/kinema:latest
    container_name: kinema
    network_mode: host # Enables UDP port 48555 LAN discovery
    restart: unless-stopped
    volumes:
      - ./data:/app/data
      - /path/to/movies:/media/movies:ro
      - /path/to/tv:/media/tv:ro
    environment:
      - KINEMA_DATA_DIR=/app/data
      - TZ=UTC`}
          </pre>
          <div className="mt-4 flex flex-wrap items-center justify-between gap-4">
            <span className="text-xs opacity-75">Spins up server with web UI on port 3000</span>
            <Link 
              to="/docs/getting-started/quick-start" 
              className="text-sm font-semibold flex items-center" 
              style={{ color: 'var(--kinema-ember)' }}
            >
              Full installation instructions <ChevronRight className="w-3.5 h-3.5 ml-1" />
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
