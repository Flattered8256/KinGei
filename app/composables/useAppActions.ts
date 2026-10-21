export type AppModal = 'group' | 'direct' | 'friend' | 'password' | null

export function useAppActions() {
  const modal = useState<AppModal>('ui.modal', () => null)

  function open(next: Exclude<AppModal, null>) {
    modal.value = next
  }

  function close() {
    modal.value = null
  }

  return {
    modal,
    open,
    close,
  }
}
