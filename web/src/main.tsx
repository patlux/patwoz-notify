import { Routes } from './Routes'
import { api } from './api'
import './index.css'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'
import ReactDOM from 'react-dom/client'

const client = new QueryClient({
  defaultOptions: {
    queries: {
      queryFn: async ({ queryKey }) => {
        const response = await api.fetchApi(`${queryKey.join('/')}`)
        if (!response.ok) {
          throw new Error(`Response not ok: "${response.status}".`)
        }
        if (response.status === 204 || response.status === 304) {
          return null
        }
        if (
          !response.headers.get('content-type')?.includes('application/json')
        ) {
          throw new Error(`Expected json as data.`)
        }
        return response.json()
      },
    },
  },
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={client}>
      <Routes />
    </QueryClientProvider>
  </React.StrictMode>,
)
