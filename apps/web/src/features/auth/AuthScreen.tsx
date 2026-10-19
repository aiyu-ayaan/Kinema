import { useState, type FormEvent, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { ArrowLeft, Backspace, LockSimple } from '@phosphor-icons/react'
import { Avatar } from '@/components/ui/Avatar'
import { Button } from '@/components/ui/Button'
import { ErrorText, Skeleton } from '@/components/ui/Card'
import { Field } from '@/components/ui/Field'
import { api } from '@/lib/api'
import { useAuth } from '@/lib/auth'
import { cn } from '@/lib/cn'
import { isDev } from '@/lib/flavor'
import { t } from '@/lib/i18n'
import { useServerInfo } from '@/lib/queries'
import type { User } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

function Backdrop({ children, wide }: { children: ReactNode; wide?: boolean }) {
  return (
    <div className="relative flex min-h-dvh items-center justify-center overflow-hidden px-4 py-10">
      {/* Soft ember glow behind the content; decoration only. */}
      <div
        aria-hidden
        className="pointer-events-none absolute -top-40 left-1/2 h-[520px] w-[820px] -translate-x-1/2 rounded-full opacity-40 blur-3xl"
        style={{ background: 'radial-gradient(closest-side, rgb(var(--primary-container)), transparent)' }}
      />
      <div className={cn('anim-enter relative w-full', wide ? 'max-w-3xl' : 'max-w-[380px]')}>{children}</div>
    </div>
  )
}

/** First run: the administrator account. */
function Setup() {
  const { signIn } = useAuth()
  const [name, setName] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const mismatch = confirm.length > 0 && confirm !== password
  const valid = name.trim().length > 0 && password.length >= 8 && confirm === password

  async function submit(e: FormEvent) {
    e.preventDefault()
    if (!valid) return
    setError(null)
    setLoading(true)
    try {
      const res = await api.setup(name.trim(), password)
      signIn(res.token, res.user)
    } catch (err) {
      setError(errorText(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <Backdrop>
      <img src={isDev ? '/favicon-dev.svg' : '/favicon.svg'} alt="" className="h-10 w-10" />
      <h1 className="mt-6 text-page font-semibold tracking-[-0.01em]">{t('auth.setup.title')}</h1>
      <p className="mt-1 text-body text-muted">{t('auth.setup.description')}</p>
      <form onSubmit={submit} className="mt-6 flex flex-col gap-4 rounded-lg border border-border bg-panel p-5 shadow-overlay">
        <Field label={t('auth.name')} autoComplete="username" required autoFocus value={name} onChange={(e) => setName(e.target.value)} />
        <Field label={t('auth.password')} type="password" autoComplete="new-password" required minLength={8} hint={t('auth.atLeast8')} value={password} onChange={(e) => setPassword(e.target.value)} />
        <Field label={t('auth.confirmPassword')} type="password" autoComplete="new-password" required error={mismatch ? t('auth.mismatch') : undefined} value={confirm} onChange={(e) => setConfirm(e.target.value)} />
        <ErrorText>{error}</ErrorText>
        <Button type="submit" variant="primary" loading={loading} disabled={!valid} className="mt-1 w-full">
          {t('auth.setup.submit')}
        </Button>
      </form>
      {typeof window !== 'undefined' && (window.kinema?.chooseServer || window.desktop?.chooseServer) && (
        <div className="mt-4 flex justify-center">
          <Button
            variant="ghost"
            onClick={() => (window.kinema?.chooseServer || window.desktop?.chooseServer)?.()}
          >
            {t('auth.changeServer')}
          </Button>
        </div>
      )}
    </Backdrop>
  )
}

const KEYS = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '', '0', 'back'] as const

/** PIN entry that works with the on-screen pad, a keyboard, or a TV remote's number keys. */
function PinPad({ profile, onBack }: { profile: User; onBack: () => void }) {
  const { signIn } = useAuth()
  const [pin, setPin] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function submit(value = pin) {
    if (value.length < 4 || loading) return
    setLoading(true)
    setError(null)
    try {
      const res = await api.login({ user_id: profile.id, pin: value })
      signIn(res.token, res.user)
    } catch (err) {
      setError(errorText(err))
      setPin('')
    } finally {
      setLoading(false)
    }
  }
  const press = (k: (typeof KEYS)[number]) => {
    if (k === 'back') setPin((p) => p.slice(0, -1))
    else if (k && pin.length < 8) setPin(pin + k)
  }

  return (
    <div
      className="flex flex-col items-center outline-none"
      tabIndex={-1}
      ref={(el) => el?.focus()}
      onKeyDown={(e) => {
        if (/^\d$/.test(e.key)) press(e.key as (typeof KEYS)[number])
        else if (e.key === 'Backspace') press('back')
        else if (e.key === 'Enter') submit()
        else if (e.key === 'Escape') onBack()
      }}
    >
      <Avatar name={profile.name} color={profile.avatar} size={88} className="shadow-overlay" />
      <h1 className="mt-4 text-page font-semibold">{profile.name}</h1>
      <p className="mt-1 text-body text-muted">{t('auth.pin.enter')}</p>
      <div className="mt-6 flex h-4 items-center gap-3" aria-live="polite" aria-label={t('auth.pin.digits', { n: pin.length })}>
        {Array.from({ length: Math.max(4, pin.length) }, (_, i) => (
          <span key={i} className={cn('h-3.5 w-3.5 rounded-full border-2 transition-colors', i < pin.length ? 'border-primary bg-primary' : 'border-border-strong')} />
        ))}
      </div>
      <div className="mt-4 min-h-10">
        <ErrorText>{error}</ErrorText>
      </div>
      <div className="mt-2 grid grid-cols-3 gap-3">
        {KEYS.map((k, i) =>
          k === '' ? (
            <span key={i} />
          ) : (
            <button
              key={i}
              type="button"
              onClick={() => press(k)}
              aria-label={k === 'back' ? t('auth.pin.delete') : k}
              className="flex h-16 w-16 cursor-pointer items-center justify-center rounded-full border border-border bg-panel text-page font-medium tabular transition-colors hover:border-border-strong hover:bg-raised active:bg-primary-container"
            >
              {k === 'back' ? <Backspace size={22} /> : k}
            </button>
          ),
        )}
      </div>
      <div className="mt-6 flex gap-2">
        <Button variant="ghost" icon={<ArrowLeft size={16} />} onClick={onBack}>
          {t('auth.profiles.back')}
        </Button>
        <Button variant="primary" loading={loading} disabled={pin.length < 4} onClick={() => submit()}>
          {t('auth.signIn.submit')}
        </Button>
      </div>
    </div>
  )
}

function PasswordForm({ profile, onBack }: { profile: User; onBack: () => void }) {
  const { signIn } = useAuth()
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function submit(e: FormEvent) {
    e.preventDefault()
    setLoading(true)
    setError(null)
    try {
      const res = await api.login({ user_id: profile.id, password })
      signIn(res.token, res.user)
    } catch (err) {
      setError(errorText(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="mx-auto flex max-w-[380px] flex-col items-center">
      <Avatar name={profile.name} color={profile.avatar} size={88} className="shadow-overlay" />
      <h1 className="mt-4 text-page font-semibold">{profile.name}</h1>
      <form onSubmit={submit} className="mt-6 flex w-full flex-col gap-4 rounded-lg border border-border bg-panel p-5 shadow-overlay">
        {/* Hidden username lets password managers pair the saved password with this profile. */}
        <input type="text" autoComplete="username" value={profile.name} readOnly hidden />
        <Field label={t('auth.password')} type="password" autoComplete="current-password" required autoFocus value={password} onChange={(e) => setPassword(e.target.value)} />
        <ErrorText>{error}</ErrorText>
        <Button type="submit" variant="primary" loading={loading} disabled={!password} className="w-full">
          {t('auth.signIn.submit')}
        </Button>
        <Button variant="ghost" icon={<ArrowLeft size={16} />} onClick={onBack} className="w-full">
          {t('auth.profiles.back')}
        </Button>
      </form>
    </div>
  )
}

/** "Who's watching?" Profiles without a lock open with one tap. */
function ProfilePicker() {
  const { signIn } = useAuth()
  const { data: info } = useServerInfo()
  const { data: profiles, isLoading } = useQuery({ queryKey: ['profiles'], queryFn: api.profiles })
  const [chosen, setChosen] = useState<User | null>(null)
  const [error, setError] = useState<string | null>(null)

  async function pick(p: User) {
    setError(null)
    if (p.has_password || p.has_pin) {
      setChosen(p)
      return
    }
    try {
      const res = await api.login({ user_id: p.id })
      signIn(res.token, res.user)
    } catch (err) {
      setError(errorText(err))
    }
  }

  if (chosen) {
    return <Backdrop>{chosen.has_pin && !chosen.has_password ? <PinPad profile={chosen} onBack={() => setChosen(null)} /> : <PasswordForm profile={chosen} onBack={() => setChosen(null)} />}</Backdrop>
  }

  return (
    <Backdrop wide>
      <div className="text-center">
        <img src={isDev ? '/favicon-dev.svg' : '/favicon.svg'} alt="" className="mx-auto h-10 w-10" />
        <h1 className="mt-6 text-hero font-bold tracking-[-0.02em]">{t('auth.profiles.title')}</h1>
        <p className="mt-2 text-body text-muted">{info?.name ?? t('app.name')}</p>
      </div>
      <div className="mt-10 flex flex-wrap justify-center gap-6">
        {isLoading &&
          Array.from({ length: 3 }, (_, i) => (
            <Skeleton key={i} className="h-36 w-28 rounded-xl" />
          ))}
        {profiles?.map((p) => (
          <button key={p.id} type="button" onClick={() => pick(p)} className="group flex w-28 cursor-pointer flex-col items-center gap-3 rounded-xl p-2 focus-visible:outline-offset-4">
            <span className="relative">
              <Avatar name={p.name} color={p.avatar} size={96} className="ring-2 ring-transparent ring-offset-4 ring-offset-bg transition-all duration-base group-hover:scale-105 group-hover:ring-fg/70" />
              {(p.has_pin || p.has_password) && (
                <span className="absolute -bottom-1 -end-1 flex h-7 w-7 items-center justify-center rounded-full border border-border bg-panel text-muted">
                  <LockSimple size={14} weight="fill" aria-label={t('auth.profiles.locked')} />
                </span>
              )}
            </span>
            <span className="max-w-full truncate text-body font-medium text-muted group-hover:text-fg">{p.name}</span>
          </button>
        ))}
      </div>
      <div className="mx-auto mt-6 max-w-sm">
        <ErrorText>{error}</ErrorText>
      </div>
      {typeof window !== 'undefined' && (window.kinema?.chooseServer || window.desktop?.chooseServer) && (
        <div className="mt-8 flex justify-center">
          <Button
            variant="ghost"
            onClick={() => (window.kinema?.chooseServer || window.desktop?.chooseServer)?.()}
          >
            {t('auth.changeServer')}
          </Button>
        </div>
      )}
    </Backdrop>
  )
}

export function AuthScreen() {
  const { needsSetup } = useAuth()
  return needsSetup ? <Setup /> : <ProfilePicker />
}
