import { Link } from 'react-router-dom'

export default function LandingPage() {
  return (
    <div className="page">
      <header className="topbar">
        <div className="brand">🔐 Auth Demo</div>
        <nav className="nav-actions">
          <Link className="btn ghost" to="/login">
            Log in
          </Link>
          <Link className="btn primary" to="/register">
            Sign up
          </Link>
        </nav>
      </header>

      <main className="container hero">
        <h1>Full-stack auth, ready to build on.</h1>
        <p className="muted lead">
          A React + TypeScript front end talking to a Rust (Axum) API, backed by
          PostgreSQL. Passwords are hashed with Argon2id and sessions ride in a
          secure, httpOnly cookie.
        </p>
        <div className="cta-row">
          <Link className="btn primary lg" to="/register">
            Create an account
          </Link>
          <Link className="btn ghost lg" to="/login">
            I already have one
          </Link>
        </div>
      </main>
    </div>
  )
}
