import { HomeRoute } from './Home/HomeRoute'
import { Root } from './Root'
import { authenticate } from './api/api'
import {
  RouterProvider,
  Router,
  Route,
  RootRoute,
} from '@tanstack/react-router'

const rootRoute = new RootRoute({
  beforeLoad: async () => {
    // TODO: catch error to display error message to the user
    await authenticate()
  },
  component: () => <Root />,
})

const indexRoute = new Route({
  getParentRoute: () => rootRoute,
  path: '/',
  component: HomeRoute,
})

const aboutRoute = new Route({
  getParentRoute: () => rootRoute,
  path: '/about',
  component: function About() {
    return <div className="p-2">Hello from About!</div>
  },
})

const routeTree = rootRoute.addChildren([indexRoute, aboutRoute])

const router = new Router({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

export const Routes = () => {
  return <RouterProvider router={router} />
}
