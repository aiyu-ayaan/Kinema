import { useEffect, useState, type FormEvent, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { ArrowLeft, Backspace, CircleNotch, HouseLine, LockSimple, ShieldWarning } from '@phosphor-icons/react'
import { Avatar } from '@/components/ui/Avatar'
import { Button } from '@/components/ui/Button'
import { ErrorText, Skeleton } from '@/components/ui/Card'
import { Field } from '@/components/ui/Field'
import { api, ApiError } from '@/lib/api'
import { useAuth } from '@/lib/auth'
import { cn } from '@/lib/cn'
import { clientDevice } from '@/lib/device'
import { isDev } from '@/lib/flavor'
import { t } from '@/lib/i18n'
import { useServerInfo } from '@/lib/queries'
import type { ApprovalRequired, ProfileCard, User } from '@/lib/types'

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e))

type Waiting = ApprovalRequired['approval_required']

/** Signs a profile in, or hands over the request the administrator has to approve first. */
function useLogin(onWaiting: (w: Waiting) => void) {
  const { signIn } = useAuth()
  return async (body: { user_id: number; password?: string; pin?: string }) => {
    const res = await api.login({ ...body, device: clientDevice() })
    if ('approval_required' in res) onWaiting(res.approval_required)
    else signIn(res.token, res.user)
  }
}

const POLL_MS = 5000

/** A new device on a profile with no free slot: waits until the administrator answers. */
function WaitingForApproval({ waiting, onBack }: { waiting: Waiting; onBack: () => void }) {
  const { signIn } = useAuth()
  const [denied, setDenied] = useState(false)

  useEffect(() => {
    if (denied) return
    let stopped = false
    const timer = setInterval(async () => {
      try {
        const res = await api.devicePoll(waiting.poll_token)
        if (stopped || 'pending' in res) return
        stopped = true
        signIn(res.token, res.user)
      } catch (err) {
        // The server drops a declined request, so its poll secret no longer exists.
        if (err instanceof ApiError && err.code === 'auth.device_denied') setDenied(true)
        // Anything else (network, restart) is retried on the next tick.
      }
    }, POLL_MS)
    return () => {
      stopped = true
      clearInterval(timer)
    }
  }, [waiting.poll_token, denied, signIn])

  return (
    <div className="flex flex-col items-center text-center">
      <span className="flex h-16 w-16 items-center justify-center rounded-full border border-border bg-panel text-primary">
        {denied ? <ShieldWarning size={30} aria-hidden /> : <CircleNotch size={30} className="animate-spin" aria-hidden />}
      </span>
      <h1 className="mt-5 text-page font-semibold">{t(denied ? 'auth.approval.deniedTitle' : 'auth.approval.title')}</h1>
      <p className="mt-2 text-body text-muted" role="status">
        {t(denied ? 'auth.approval.denied' : 'auth.approval.description', { device: waiting.device_name })}
      </p>
      <Button variant="ghost" icon={<ArrowLeft size={16} />} onClick={onBack} className="mt-6">
        {t(denied ? 'auth.profiles.back' : 'common.cancel')}
      </Button>
    </div>
  )
}

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
      const res = await api.setup(name.trim(), password, clientDevice())
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
function PinPad({ profile, onBack, onWaiting }: { profile: User; onBack: () => void; onWaiting: (w: Waiting) => void }) {
  const login = useLogin(onWaiting)
  const [pin, setPin] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function submit(value = pin) {
    if (value.length < 4 || loading) return
    setLoading(true)
    setError(null)
    try {
      await login({ user_id: profile.id, pin: value })
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

function PasswordForm({ profile, onBack, onWaiting }: { profile: User; onBack: () => void; onWaiting: (w: Waiting) => void }) {
  const login = useLogin(onWaiting)
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function submit(e: FormEvent) {
    e.preventDefault()
    setLoading(true)
    setError(null)
    try {
      await login({ user_id: profile.id, password })
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
  const { data: info } = useServerInfo()
  const { data: profiles, isLoading } = useQuery({ queryKey: ['profiles'], queryFn: api.profiles })
  const [chosen, setChosen] = useState<User | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [waiting, setWaiting] = useState<Waiting | null>(null)
  const login = useLogin(setWaiting)

  async function pick(p: ProfileCard) {
    setError(null)
    if (p.home_only) {
      setError(t('auth.profiles.homeOnlyHint', { name: p.name }))
      return
    }
    // The server says whether this device may skip the lock (home network); otherwise ask for it.
    if (!p.opens_without_lock) {
      setChosen(p)
      return
    }
    try {
      await login({ user_id: p.id })
    } catch (err) {
      setError(errorText(err))
    }
  }

  if (waiting) {
    return (
      <Backdrop>
        <WaitingForApproval
          waiting={waiting}
          onBack={() => {
            setWaiting(null)
            setChosen(null)
          }}
        />
      </Backdrop>
    )
  }

  if (chosen) {
    return (
      <Backdrop>
        {chosen.has_pin && !chosen.has_password ? (
          <PinPad profile={chosen} onBack={() => setChosen(null)} onWaiting={setWaiting} />
        ) : (
          <PasswordForm profile={chosen} onBack={() => setChosen(null)} onWaiting={setWaiting} />
        )}
      </Backdrop>
    )
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
          <button
            key={p.id}
            type="button"
            onClick={() => pick(p)}
            aria-disabled={p.home_only || undefined}
            aria-describedby={p.home_only ? `home-only-${p.id}` : undefined}
            className={cn('group flex w-28 cursor-pointer flex-col items-center gap-3 rounded-xl p-2 focus-visible:outline-offset-4', p.home_only && 'cursor-not-allowed opacity-60')}
          >
            <span className="relative">
              <Avatar name={p.name} color={p.avatar} size={96} className="ring-2 ring-transparent ring-offset-4 ring-offset-bg transition-all duration-base group-hover:scale-105 group-hover:ring-fg/70" />
              {p.home_only && (
                <span className="absolute -bottom-1 -end-1 flex h-7 w-7 items-center justify-center rounded-full border border-border bg-panel text-muted">
                  <HouseLine size={14} weight="fill" aria-hidden />
                </span>
              )}
              {!p.opens_without_lock && !p.home_only && (
                <span className="absolute -bottom-1 -end-1 flex h-7 w-7 items-center justify-center rounded-full border border-border bg-panel text-muted">
                  <LockSimple size={14} weight="fill" aria-label={t('auth.profiles.locked')} />
                </span>
              )}
            </span>
            <span className="max-w-full truncate text-body font-medium text-muted group-hover:text-fg">{p.name}</span>
            {p.home_only && (
              <span id={`home-only-${p.id}`} className="text-caption text-subtle">
                {t('auth.profiles.homeOnly')}
              </span>
            )}
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
