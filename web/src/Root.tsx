import { Outlet, Link } from '@tanstack/react-router'
import { TanStackRouterDevtools } from '@tanstack/router-devtools'

export const Root = () => {
  return (
    <>
      <div className="w-full bg-brand-50">
        <div className="px-2 container mx-auto flex-1 py-4 flex flex-row justify-between items-center">
          <div className="font-semibold font-mono">patwoz-notify</div>
        </div>
      </div>
      <NavMenu />
      <Outlet />
      {process.env.NODE_ENV === 'development' && <TanStackRouterDevtools />}
    </>
  )
}

function NavMenu() {
  return (
    <div className="sticky top-0 w-full bg-brand-50 border-b-2 border-brand-100/50 z-50">
      <div className="container mx-auto flex-1 flex flex-row justify-between items-center">
        <nav className="w-full font-semibold flex flex-row justify-between -mb-[2px] overflow-y-auto space-x-6">
          <ul className="pl-2 flex flex-row space-x-6">
            <li>
              <Link className="block" to="/">
                {({ isActive }) => (
                  <span
                    className={`block relative pb-2 border-b-2 ${isActive ? 'border-brand-500 text-brand-700' : 'border-b-transparent'}`}
                  >
                    Home
                  </span>
                )}
              </Link>
            </li>
          </ul>
        </nav>
      </div>
    </div>
  )
}
