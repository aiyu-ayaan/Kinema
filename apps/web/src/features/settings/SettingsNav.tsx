import { NavLink } from 'react-router-dom'
import { ArrowLeft } from '@phosphor-icons/react'
import { usePendingDevices } from '@/features/devices/usePendingDevices'
import { cn } from '@/lib/cn'
import { t } from '@/lib/i18n'
import { settingsGroups } from './sections'

const itemCls = (active: boolean) =>
  cn(
    'flex h-9 items-center gap-3 rounded-md px-3 text-body font-medium transition-colors duration-fast',
    active ? 'bg-primary-container text-on-primary-container' : 'text-muted hover:bg-raised hover:text-fg',
  )

export function CountBadge({ n }: { n: number }) {
  return (
    <span className="ms-auto flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 text-caption font-semibold tabular text-on-primary" aria-label={t('devices.pendingCount', { n })}>
      {n}
    </span>
  )
}

/** Replaces the library list in the app sidebar while Settings is open. */
export function SettingsNav({ onNavigate }: { onNavigate?: () => void }) {
  const pendingDevices = usePendingDevices()
  return (
    <nav aria-label={t('settings.title')} className="flex flex-1 flex-col gap-1 overflow-y-auto px-3 pb-3">
      <NavLink to="/" end onClick={onNavigate} className={itemCls(false)}>
        <ArrowLeft size={18} aria-hidden />
        {t('settings.backHome')}
      </NavLink>
      {settingsGroups.map((g) => (
        <div key={g.label} className="mt-4">
          <p className="px-3 pb-1 text-caption font-semibold uppercase tracking-wider text-subtle">{t(g.label)}</p>
          <div className="flex flex-col gap-0.5">
            {g.sections.map((s) => {
              const I = s.icon
              return (
                <NavLink key={s.path} to={`/settings/${s.path}`} onClick={onNavigate} className={({ isActive }) => itemCls(isActive)}>
                  {({ isActive }) => (
                    <>
                      <I size={18} weight={isActive ? 'fill' : 'regular'} aria-hidden />
                      <span className="truncate">{t(s.label)}</span>
                      {s.path === 'devices' && pendingDevices > 0 && <CountBadge n={pendingDevices} />}
                    </>
                  )}
                </NavLink>
              )
            })}
          </div>
        </div>
      ))}
    </nav>
  )
}
