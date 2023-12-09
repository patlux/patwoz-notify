import { SubscriptionCard } from './SubscriptionCard'

function App() {
  return (
    <div className="h-screen bg-zinc-800 px-8">
      <div className="sm:mx-auto sm:container">
        <h1 className="py-8 text-white font-semibold text-center text-lg">
          patwoz-notify
        </h1>
        <div className="sm:w-1/3">
          <SubscriptionCard />
        </div>
      </div>
    </div>
  )
}

export default App
