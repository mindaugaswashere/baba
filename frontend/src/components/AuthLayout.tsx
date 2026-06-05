import type { ReactNode } from 'react'
import { Link } from 'react-router-dom'

interface Props {
  title: string
  subtitle: string
  children: ReactNode
}

export default function AuthLayout({ title, subtitle, children }: Props) {
  return (
    <div className="auth-page">
      <div className="auth-card">
        <Link to="/" className="brand center">
          🔐 Auth Demo
        </Link>
        <h1>{title}</h1>
        <p className="muted center">{subtitle}</p>
        {children}
      </div>
    </div>
  )
}
