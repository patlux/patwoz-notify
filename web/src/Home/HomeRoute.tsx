import { SubscriptionCard } from './SubscriptionCard'

export const HomeRoute = () => {
  return (
    <div className="container mx-auto pt-4 pb-24">
      <div className="grid grid-cols-2 row-gap-8 md:grid-cols-3">
        <SubscriptionCard />
      </div>
    </div>
  )
}
