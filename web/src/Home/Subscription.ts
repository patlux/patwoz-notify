import { api } from '../api'
import { useCallback, useEffect, useRef, useState } from 'react'

export function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4)
  const base64 = (base64String + padding).replace(/\-/g, '+').replace(/_/g, '/')
  const rawData = atob(base64)
  const outputArray = new Uint8Array(rawData.length)
  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i)
  }
  return outputArray
}

export const useServiceWorker = () => {
  const swRef = useRef<ServiceWorkerRegistration>()
  const [isReady, setReady] = useState(false)

  useEffect(() => {
    const registerSw = async () => {
      console.log(`Register..`, '/sw.ts')
      try {
        if ('serviceWorker' in navigator) {
          swRef.current = await navigator.serviceWorker.register(
            process.env.NODE_ENV === 'production' ? '/sw.js' : '/sw.ts',
            {
              // scope: './',
              updateViaCache: 'all',
            },
          )
          console.log(`Registered.`)
          setReady(true)
        }
      } catch (error: unknown) {
        console.error(error)
      }
    }
    registerSw()
  }, [])

  return {
    isReady,
    swRef,
  }
}

export const useSubscriptionPermission = ({
  swRef,
  isReady,
}: ReturnType<typeof useServiceWorker>) => {
  const [permissionState, setPermissionState] =
    useState<PermissionState | null>(null)

  const checkPermission = useCallback(async () => {
    const pushManager = swRef.current!.pushManager

    const state = await pushManager.permissionState({
      applicationServerKey: urlBase64ToUint8Array(
        await api.getVapidPublicKey(),
      ),
      userVisibleOnly: true,
    })

    setPermissionState(state ?? null)
  }, [])

  const askForPermission = useCallback(async () => {
    await window.Notification.requestPermission()
    await checkPermission()
  }, [checkPermission])

  useEffect(() => {
    if (!isReady) return
    checkPermission().catch((error) => {
      console.error(`ERR from checkPermission(): "${error}".`)
    })
  }, [isReady])

  return {
    check: checkPermission,
    state: permissionState,
    askFor: askForPermission,
  }
}

export const useSubscription = ({
  swRef,
  isReady,
}: ReturnType<typeof useServiceWorker>) => {
  const [subscription, setSubscription] = useState<PushSubscription | null>(
    null,
  )

  useEffect(() => {
    if (!isReady) return

    const getInitialSubscription = async () => {
      const pushManager = swRef.current!.pushManager
      setSubscription(await pushManager.getSubscription())
    }

    getInitialSubscription()
  }, [isReady])

  const subscribe = useCallback(async () => {
    const pushManager = swRef.current!.pushManager
    const subscription = await pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: urlBase64ToUint8Array(
        await api.getVapidPublicKey(),
      ),
    })

    await api.subscribe(subscription)
    setSubscription(subscription)
  }, [])

  useEffect(() => {
    const sendSubscription = async () => {
      if (subscription == null) return
      const response = await api.subscribe(subscription)
      if (response.status !== 304 && response.status !== 200) {
        throw new Error(`Server response not ok.`)
      }
    }

    sendSubscription()
  }, [subscription])

  return {
    subscription,
    subscribe,
  }
}
