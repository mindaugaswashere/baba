import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../auth/AuthContext'
import { ApiError } from '../api/client'
import AuthLayout from '../components/AuthLayout'

const schema = z.object({
  email: z.email('Enter a valid email address'),
  password: z.string().min(1, 'Password is required'),
})

type FormValues = z.infer<typeof schema>

export default function LoginPage() {
  const { login } = useAuth()
  const navigate = useNavigate()
  const [serverError, setServerError] = useState<string | null>(null)
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({ resolver: zodResolver(schema) })

  const onSubmit = async (values: FormValues) => {
    setServerError(null)
    try {
      await login(values.email, values.password)
      navigate('/', { replace: true })
    } catch (err) {
      setServerError(
        err instanceof ApiError ? err.message : 'Something went wrong. Please try again.',
      )
    }
  }

  return (
    <AuthLayout title="Welcome back" subtitle="Log in to your account">
      <form onSubmit={handleSubmit(onSubmit)} noValidate>
        {serverError && <div className="alert">{serverError}</div>}

        <label className="field">
          <span>Email</span>
          <input type="email" autoComplete="email" {...register('email')} />
          {errors.email && <small className="error">{errors.email.message}</small>}
        </label>

        <label className="field">
          <span>Password</span>
          <input type="password" autoComplete="current-password" {...register('password')} />
          {errors.password && <small className="error">{errors.password.message}</small>}
        </label>

        <button className="btn primary block" type="submit" disabled={isSubmitting}>
          {isSubmitting ? 'Logging in…' : 'Log in'}
        </button>
      </form>

      <p className="muted center">
        No account? <Link to="/register">Create one</Link>
      </p>
    </AuthLayout>
  )
}
