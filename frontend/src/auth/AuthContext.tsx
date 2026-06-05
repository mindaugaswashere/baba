import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from 'react'
import type { User } from '../types'
import * as api from '../api/client'

type Status = 'loading' | 'authenticated' | 'guest'

interface AuthContextValue {
  user: User | null
  status: Status
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string) => Promise<void>
  logout: () => Promise<void>
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [status, setStatus] = useState<Status>('loading')

  // On first load, ask the API whether the session cookie is still valid.
  useEffect(() => {
    api
      .me()
      .then((res) => {
        setUser(res.user)
        setStatus('authenticated')
      })
      .catch(() => {
        setUser(null)
        setStatus('guest')
      })
  }, [])

  const login = async (email: string, password: string) => {
    const res = await api.login(email, password)
    setUser(res.user)
    setStatus('authenticated')
  }

  const register = async (email: string, password: string) => {
    const res = await api.register(email, password)
    setUser(res.user)
    setStatus('authenticated')
  }

  const logout = async () => {
    await api.logout()
    setUser(null)
    setStatus('guest')
  }

  return (
    <AuthContext.Provider value={{ user, status, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return ctx
}
