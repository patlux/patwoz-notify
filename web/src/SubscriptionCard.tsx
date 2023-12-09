import { useQuery } from 'react-query'
import {
  useSubscriptionPermission,
  useServiceWorker,
  useSubscription,
} from './Subscription'
import * as api from './api'

export const SubscriptionCard = () => {
  const sw = useServiceWorker()
  const permission = useSubscriptionPermission(sw)
  const { subscription, subscribe } = useSubscription(sw)

  const subscriptions = useQuery('/subscriptions')

  return (
    <div className="p-4 rounded-xl w-full aspect-square bg-white/5">
      <h3 className="font-light text-white text-lg">
        Subscriptions ({subscriptions.data?.subscriptions?.length})
      </h3>
      {!sw.isReady ? (
        <>Loading...</>
      ) : (
        <>
          <p className="font-semibold text-white text-lg">
            Permission: {permission.state ?? 'n/a'}
          </p>
          {subscription != null && (
            <button
              type="button"
              className="inline-flex items-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
              onClick={async () => {
                try {
                  const subscription =
                    await sw.swRef.current!.pushManager.getSubscription()

                  if (subscription == null) {
                    throw new Error(`Missing subscription.`)
                  }

                  api.send(subscription)
                } catch (error: unknown) {
                  alert(`${error}`)
                }
              }}
            >
              Send Notification
            </button>
          )}
          {permission.state === 'granted' && subscription == null && (
            <>
              <button
                type="button"
                className="inline-flex items-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                onClick={async () => {
                  try {
                    await subscribe()
                  } catch (error: unknown) {
                    alert(`${error}`)
                  }
                }}
              >
                Subscribe
              </button>
            </>
          )}
          {permission.state !== 'granted' && (
            <>
              <button
                type="button"
                className="inline-flex items-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                onClick={async () => {
                  try {
                    await permission.askFor()
                  } catch (error: unknown) {
                    alert(`${error}`)
                  }
                }}
              >
                Ask for permission
              </button>
            </>
          )}
        </>
      )}
    </div>
  )
}
