import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../auth/AuthContext'
import { ApiError } from '../api/client'
import AuthLayout from '../components/AuthLayout'

const schema = z
  .object({
    email: z.email('Enter a valid email address'),
    password: z.string().min(8, 'Password must be at least 8 characters'),
    confirmPassword: z.string(),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: 'Passwords do not match',
    path: ['confirmPassword'],
  })

type FormValues = z.infer<typeof schema>

export default function RegisterPage() {
  const { register: registerUser } = useAuth()
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
      await registerUser(values.email, values.password)
      navigate('/', { replace: true })
    } catch (err) {
      setServerError(
        err instanceof ApiError ? err.message : 'Something went wrong. Please try again.',
      )
    }
  }

  return (
    <AuthLayout title="Create your account" subtitle="It only takes a moment">
      <form onSubmit={handleSubmit(onSubmit)} noValidate>
        {serverError && <div className="alert">{serverError}</div>}

        <label className="field">
          <span>Email</span>
          <input type="email" autoComplete="email" {...register('email')} />
          {errors.email && <small className="error">{errors.email.message}</small>}
        </label>

        <label className="field">
          <span>Password</span>
          <input type="password" autoComplete="new-password" {...register('password')} />
          {errors.password && <small className="error">{errors.password.message}</small>}
        </label>

        <label className="field">
          <span>Confirm password</span>
          <input type="password" autoComplete="new-password" {...register('confirmPassword')} />
          {errors.confirmPassword && (
            <small className="error">{errors.confirmPassword.message}</small>
          )}
        </label>

        <button className="btn primary block" type="submit" disabled={isSubmitting}>
          {isSubmitting ? 'Creating account…' : 'Create account'}
        </button>
      </form>

      <p className="muted center">
        Already have an account? <Link to="/login">Log in</Link>
      </p>
    </AuthLayout>
  )
}
