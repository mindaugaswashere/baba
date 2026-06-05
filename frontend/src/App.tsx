import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { AuthProvider, useAuth } from './auth/AuthContext'
import type { ReactNode } from 'react'
import LandingPage from './pages/LandingPage'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'
import DashboardPage from './pages/DashboardPage'
import Spinner from './components/Spinner'

// `/` is the same path for everyone: guests get the landing page, authenticated
// users get their dashboard.
function RootPage() {
  const { status, user } = useAuth()
  if (status === 'loading') return <Spinner />
  return user ? <DashboardPage /> : <LandingPage />
}

// Routes that only make sense when logged out (login / register). Already
// authenticated users are bounced to the dashboard.
function GuestOnly({ children }: { children: ReactNode }) {
  const { status, user } = useAuth()
  if (status === 'loading') return <Spinner />
  return user ? <Navigate to="/" replace /> : <>{children}</>
}

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<RootPage />} />
          <Route
            path="/login"
            element={
              <GuestOnly>
                <LoginPage />
              </GuestOnly>
            }
          />
          <Route
            path="/register"
            element={
              <GuestOnly>
                <RegisterPage />
              </GuestOnly>
            }
          />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  )
}
