import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { api, session, setUnauthorizedHandler } from './api'
import type { User } from './types'

interface AuthState {
  user: User | null
  /** null while checking whether first-run setup is needed */
  needsSetup: boolean | null
  loading: boolean
  signIn: (token: string, user: User) => void
  signOut: () => void
}

const AuthContext = createContext<AuthState | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const qc = useQueryClient()
  const [user, setUser] = useState<User | null>(null)
  const [needsSetup, setNeedsSetup] = useState<boolean | null>(null)
  const [loading, setLoading] = useState(true)

  const signOut = useCallback(() => {
    api.logout().catch(() => {})
    session.set(null)
    setUser(null)
    qc.clear()
  }, [qc])

  useEffect(() => {
    setUnauthorizedHandler(() => {
      session.set(null)
      setUser(null)
    })
    let cancelled = false
    ;(async () => {
      try {
        const status = await api.authStatus()
        if (cancelled) return
        setNeedsSetup(status.needs_setup)
        if (!status.needs_setup && session.token) setUser(await api.me())
      } catch {
        if (!cancelled) setNeedsSetup(false)
      } finally {
        if (!cancelled) setLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [])

  const value = useMemo<AuthState>(
    () => ({
      user,
      needsSetup,
      loading,
      signIn: (token, u) => {
        session.set(token)
        setNeedsSetup(false)
        setUser(u)
      },
      signOut,
    }),
    [user, needsSetup, loading, signOut],
  )
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth outside AuthProvider')
  return ctx
}

// eslint-disable-next-line react-refresh/only-export-components
export const useIsAdmin = () => useAuth().user?.is_admin === true
