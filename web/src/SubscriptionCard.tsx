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

  const subscriptions = useQuery('/subscriptions', {
    queryFn: api.getSubscriptions,
  })

  return (
    <div className="p-4 rounded-xl w-full aspect-square bg-white/5">
      <h3 className="font-light text-white text-lg mb-4">
        Subscriptions ({subscriptions.data?.subscriptions?.length})
      </h3>
      {!sw.isReady ? (
        <>Loading...</>
      ) : (
        <>
          {permission.state === 'granted' && subscription != null ? (
            <>
              <button
                type="button"
                className="mb-4 inline-flex items-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                onClick={async () => {
                  try {
                    const subscription =
                      await sw.swRef.current!.pushManager.getSubscription()

                    if (subscription == null) {
                      throw new Error(`Missing subscription.`)
                    }

                    await api.send(subscription, {
                      title: 'JS',
                      body: 'Hello JS World!',
                    })
                  } catch (error: unknown) {
                    alert(`${error}`)
                  }
                }}
              >
                Send Test Notification
              </button>
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

                    await api.sendToAll({
                      title: 'JS',
                      body: 'Hello JS World!',
                    })
                  } catch (error: unknown) {
                    alert(`${error}`)
                  }
                }}
              >
                Send Test Notification To All
              </button>
            </>
          ) : permission.state === 'granted' && subscription == null ? (
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
          ) : permission.state !== 'granted' ? (
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
          ) : null}
        </>
      )}
    </div>
  )
}
