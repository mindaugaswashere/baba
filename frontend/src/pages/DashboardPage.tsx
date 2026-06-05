import { useState } from 'react'
import { useAuth } from '../auth/AuthContext'

export default function DashboardPage() {
  const { user, logout } = useAuth()
  const [loggingOut, setLoggingOut] = useState(false)

  if (!user) return null

  const memberSince = new Date(user.createdAt).toLocaleString()

  const handleLogout = async () => {
    setLoggingOut(true)
    try {
      await logout()
    } finally {
      setLoggingOut(false)
    }
  }

  return (
    <div className="page">
      <header className="topbar">
        <div className="brand">🔐 Auth Demo</div>
        <div className="nav-actions">
          <span className="muted email-pill">{user.email}</span>
          <button className="btn ghost" onClick={handleLogout} disabled={loggingOut}>
            {loggingOut ? 'Logging out…' : 'Log out'}
          </button>
        </div>
      </header>

      <main className="container">
        <h1>You&apos;re logged in 🎉</h1>
        <p className="muted lead">
          This authenticated view is served at <code>/</code> — guests see the
          landing page at the same path, you see this.
        </p>

        <div className="card">
          <h2>Your account</h2>
          <table className="user-table">
            <tbody>
              <tr>
                <th>User ID</th>
                <td>
                  <code>{user.id}</code>
                </td>
              </tr>
              <tr>
                <th>Email</th>
                <td>{user.email}</td>
              </tr>
              <tr>
                <th>Member since</th>
                <td>{memberSince}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </main>
    </div>
  )
}
