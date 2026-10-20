import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { ArrowLeft, ArrowSquareOut, CaretRight, GithubLogo, Globe } from '@phosphor-icons/react'
import { Card, CardHeader, PageHeader } from '@/components/ui/Card'
import { DEVELOPER, REPO, REPO_URL, credits } from '@/lib/credits'
import { isDesktop } from '@/lib/desktop'
import { isDev } from '@/lib/flavor'
import { useServerInfo } from '@/lib/queries'
import { t } from '@/lib/i18n'

const row = 'flex min-h-12 items-center justify-between gap-3 px-4 py-2 transition-colors duration-fast hover:bg-raised'

function Ext({ href, icon, title, subtitle }: { href: string; icon?: ReactNode; title: string; subtitle?: string }) {
  return (
    <a href={href} target="_blank" rel="noreferrer" className={row}>
      <span className="flex items-center gap-3">
        {icon}
        <span>
          <span className="block">{title}</span>
          {subtitle && <span className="block text-label text-muted">{subtitle}</span>}
        </span>
      </span>
      <ArrowSquareOut size={16} className="text-subtle" aria-hidden />
    </a>
  )
}

function BackLink({ to, label }: { to: string; label: string }) {
  return (
    <Link to={to} className="mb-4 inline-flex items-center gap-1.5 text-label text-muted hover:text-fg">
      <ArrowLeft size={14} className="rtl:-scale-x-100" aria-hidden /> {label}
    </Link>
  )
}

export function AboutPage() {
  const { data: info } = useServerInfo()
  return (
    <div className="mx-auto max-w-2xl">
      <PageHeader title={t('about.title')} description={t('about.description')} />
      <div className="grid gap-6">
        <Card>
          <CardHeader title={t('about.app')} />
          <div className="divide-y divide-border">
            {[
              [t('about.version'), info?.version ?? __APP_VERSION__],
              [t('about.build'), String(__APP_BUILD__)],
              [t('about.environment'), isDev ? t('about.env.dev') : t('about.env.prod')],
              [t('about.platform'), isDesktop ? t('about.platform.desktop') : t('about.platform.web')],
              [t('about.privacy'), t('about.privacyValue')],
            ].map(([k, v]) => (
              <div key={k} className="flex min-h-12 items-center justify-between gap-4 px-4 py-2">
                <span className="text-muted">{k}</span>
                <span className="text-end">{v}</span>
              </div>
            ))}
            <Ext href={REPO_URL} icon={<GithubLogo size={18} aria-hidden />} title={t('about.sourceCode')} subtitle={`github.com/${REPO}`} />
          </div>
        </Card>

        <Card>
          <CardHeader title={t('about.developer')} />
          <div className="flex flex-col items-center gap-1 px-4 py-6">
            <img src="/developer.jpg" alt={DEVELOPER.name} width={80} height={80} className="h-20 w-20 rounded-full border border-border object-cover" />
            <p className="mt-2 text-title font-medium">{DEVELOPER.name}</p>
            <p className="text-muted">{t('about.developerNote')}</p>
          </div>
          <div className="divide-y divide-border border-t border-border">
            <Ext href={DEVELOPER.github} icon={<GithubLogo size={18} aria-hidden />} title="GitHub" subtitle="github.com/aiyu-ayaan" />
            <Ext href={DEVELOPER.site} icon={<Globe size={18} aria-hidden />} title={t('about.portfolio')} subtitle="aiyu.co.in" />
          </div>
        </Card>

        <Card>
          <CardHeader title={t('about.openSource')} />
          <Link to="/about/acknowledgements" className={row}>
            <span>
              <span className="block">{t('about.acknowledgements')}</span>
              <span className="block text-label text-muted">{t('about.acknowledgementsHint')}</span>
            </span>
            <CaretRight size={16} className="text-subtle rtl:-scale-x-100" aria-hidden />
          </Link>
        </Card>
      </div>
    </div>
  )
}

export function AcknowledgementsPage() {
  return (
    <div className="mx-auto max-w-2xl">
      <BackLink to="/about" label={t('about.title')} />
      <PageHeader title={t('about.acknowledgements')} description={t('about.acknowledgementsDescription')} />
      <div className="grid gap-6">
        {credits.map((g) => (
          <Card key={g.group}>
            <CardHeader title={g.group} />
            <div className="divide-y divide-border">
              {g.items.map((c) => (
                <Ext key={c.name} href={c.url} title={c.name} subtitle={c.license} />
              ))}
            </div>
          </Card>
        ))}
      </div>
    </div>
  )
}
