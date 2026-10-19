import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { App } from './App'
import { AuthProvider } from './lib/auth'
import { initializeFlavor } from './lib/flavor'
import { applyStoredTheme } from './lib/theme'
import './index.css'

initializeFlavor()
applyStoredTheme()

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Progress made on another device (the Android app) shows up when this tab comes back.
      refetchOnWindowFocus: true,
      staleTime: 1000 * 30,
      retry: 1,
    },
  },
})

const root = document.getElementById('root')
if (root) {
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AuthProvider>
            <App />
          </AuthProvider>
        </BrowserRouter>
      </QueryClientProvider>
    </React.StrictMode>,
  )
}
