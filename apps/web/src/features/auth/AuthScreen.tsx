import { useState, type FormEvent } from 'react'
import { Button } from '@/components/ui/Button'
import { ErrorText } from '@/components/ui/Card'
import { Field } from '@/components/ui/Field'
import { api } from '@/lib/api'
import { useAuth } from '@/lib/auth'
import { useServerInfo } from '@/lib/queries'
import { isDev } from '@/lib/flavor'
import { t } from '@/lib/i18n'

/** First run creates the administrator; afterwards it is the sign-in form. */
export function AuthScreen() {
  const { needsSetup, signIn } = useAuth()
  const { data: info } = useServerInfo()
  const setup = !!needsSetup
  const [name, setName] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const mismatch = setup && confirm.length > 0 && confirm !== password
  const valid = name.trim().length > 0 && (setup ? password.length >= 8 && confirm === password : password.length > 0)

  async function submit(e: FormEvent) {
    e.preventDefault()
    if (!valid) return
    setError(null)
    setLoading(true)
    try {
      const res = setup ? await api.setup(name.trim(), password) : await api.login(name.trim(), password)
      signIn(res.token, res.user)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="relative flex min-h-dvh items-center justify-center overflow-hidden px-4 py-10">
      {/* Soft ember glow behind the card; decoration only. */}
      <div
        aria-hidden
        className="pointer-events-none absolute -top-40 left-1/2 h-[520px] w-[820px] -translate-x-1/2 rounded-full opacity-40 blur-3xl"
        style={{ background: 'radial-gradient(closest-side, rgb(var(--primary-container)), transparent)' }}
      />
      <div className="anim-enter relative w-full max-w-[380px]">
        <img src={isDev ? '/favicon-dev.svg' : '/favicon.svg'} alt="" className="h-10 w-10" />
        <h1 className="mt-6 text-page font-semibold tracking-[-0.01em]">{t(setup ? 'auth.setup.title' : 'auth.signIn.title')}</h1>
        <p className="mt-1 text-body text-muted">{setup ? t('auth.setup.description') : t('auth.signIn.description', { name: info?.name ?? t('app.name') })}</p>
        <form onSubmit={submit} className="mt-6 flex flex-col gap-4 rounded-lg border border-border bg-panel p-5 shadow-overlay">
          <Field label={t('auth.name')} autoComplete="username" required autoFocus value={name} onChange={(e) => setName(e.target.value)} />
          <Field
            label={t('auth.password')}
            type="password"
            autoComplete={setup ? 'new-password' : 'current-password'}
            required
            minLength={setup ? 8 : undefined}
            hint={setup ? t('auth.atLeast8') : undefined}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
          {setup && (
            <Field
              label={t('auth.confirmPassword')}
              type="password"
              autoComplete="new-password"
              required
              error={mismatch ? t('auth.mismatch') : undefined}
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
            />
          )}
          <ErrorText>{error}</ErrorText>
          <Button type="submit" variant="primary" loading={loading} disabled={!valid} className="mt-1 w-full">
            {t(setup ? 'auth.setup.submit' : 'auth.signIn.submit')}
          </Button>
        </form>
      </div>
    </div>
  )
}
