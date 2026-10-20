import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { ArrowLeft, ArrowSquareOut, CaretRight, GithubLogo, Globe } from '@phosphor-icons/react'
import { UpdateProgress, useDesktopUpdate } from '@/components/UpdatePrompt'
import { Button } from '@/components/ui/Button'
import { Card, CardHeader, ErrorText, PageHeader } from '@/components/ui/Card'
import { useIsAdmin } from '@/lib/auth'
import { DEVELOPER, REPO, REPO_URL, credits } from '@/lib/credits'
import { isDesktop } from '@/lib/desktop'
import { isDev } from '@/lib/flavor'
import { useServerInfo } from '@/lib/queries'
import { updateCommands, useServerUpdate } from '@/lib/releases'
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

/** Admins only: whether GitHub has a newer server release, and the command that installs it. */
function ServerUpdateCard() {
  const admin = useIsAdmin()
  const running = useServerInfo().data?.version ?? __APP_VERSION__
  const check = useServerUpdate(admin)
  const found = check.data
  if (!admin) return null
  return (
    <Card>
      <CardHeader
        title={t('update.title')}
        action={
          found && (
            <a href={found.url} target="_blank" rel="noreferrer" className="inline-flex items-center gap-1 text-label text-primary hover:underline">
              {t('update.release')} <ArrowSquareOut size={14} aria-hidden />
            </a>
          )
        }
      />
      <div className="grid gap-3 p-4">
        <p className={found ? 'font-medium text-primary' : 'text-muted'}>
          {check.isError
            ? t('update.checkFailed')
            : check.isPending
              ? t('update.checking')
              : found
                ? t('update.available', { version: found.version })
                : t('update.upToDate', { version: running })}
        </p>
        {found && (
          <>
            <p className="text-label text-muted">{t('update.youHave', { version: running })}</p>
            {updateCommands(found.channel).map((c) => (
              <div key={c.key}>
                <p className="mb-1 text-label font-medium">{t(`update.cmd.${c.key}`)}</p>
                {/* select-all: one click selects the whole command for copying. */}
                <pre className="select-all overflow-x-auto rounded-md border border-border bg-raised px-3 py-2 font-mono text-label">{c.command}</pre>
              </div>
            ))}
            <p className="text-label text-muted">{t('update.commandHint')}</p>
          </>
        )}
      </div>
    </Card>
  )
}

/** Desktop app only: its own version and update, next to the server's. */
function DesktopUpdateCard() {
  const { info, check, install, progress, busy } = useDesktopUpdate()
  if (!info) return null
  const found = check.data
  return (
    <Card>
      <CardHeader title={t('desktopUpdate.title')} />
      <div className="grid gap-3 p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className={found ? 'font-medium text-primary' : 'text-muted'}>
            {check.isError
              ? t('update.checkFailed')
              : check.isFetching
                ? t('update.checking')
                : found
                  ? t('desktopUpdate.available', { version: found.version })
                  : t('desktopUpdate.upToDate', { version: info.version })}
          </p>
          {found ? (
            <Button size="sm" variant="primary" loading={busy} onClick={() => install.mutate()}>
              {info.selfInstall ? t('desktopUpdate.restart') : t('desktopUpdate.download')}
            </Button>
          ) : (
            <Button size="sm" variant="ghost" disabled={check.isFetching} onClick={() => check.refetch()}>
              {t('desktopUpdate.checkNow')}
            </Button>
          )}
        </div>
        {progress !== null && <UpdateProgress value={progress} />}
        <ErrorText>{install.data}</ErrorText>
      </div>
    </Card>
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

        <ServerUpdateCard />
        <DesktopUpdateCard />

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
