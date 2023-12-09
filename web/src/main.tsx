import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'
import { QueryClient, QueryClientProvider } from 'react-query'
import { fetchApi } from './api.ts'

const client = new QueryClient({
  defaultOptions: {
    queries: {
      queryFn: async ({ queryKey }) => {
        const response = await fetchApi(`${queryKey.join('/')}`)
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
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
)
