import type { User } from '../types'

// All requests go to /api, which Vite proxies to the Rust backend (see
// vite.config.ts). `credentials: 'include'` makes the browser send/receive the
// httpOnly session cookie.
const API_BASE = '/api'

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...(options.headers ?? {}),
    },
    ...options,
  })

  if (res.status === 204) {
    return undefined as T
  }

  const data = await res.json().catch(() => null)

  if (!res.ok) {
    const message =
      data && typeof data.error === 'string'
        ? data.error
        : `Request failed (${res.status})`
    throw new ApiError(res.status, message)
  }

  return data as T
}

interface AuthResponse {
  user: User
}

export function register(email: string, password: string) {
  return request<AuthResponse>('/auth/register', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  })
}

export function login(email: string, password: string) {
  return request<AuthResponse>('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  })
}

export function logout() {
  return request<void>('/auth/logout', { method: 'POST' })
}

export function me() {
  return request<AuthResponse>('/auth/me')
}
